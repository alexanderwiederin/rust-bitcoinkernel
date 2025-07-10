#include "arith_uint256.h"
#include "chain.h"
#include "chainparams.h"
#include "logging.h"
#include "sync.h"
#include "util/signalinterrupt.h"
#include "util/translation.h"
#include "validation.h"
#include <kernel/chainparams.h>
#include <kernel/notifications_interface.h>
#include <node/blockstorage.h>

#include <memory>
#include <optional>

enum class IBDStatus {
    IN_IBD,
    SYNCED,
    NO_DATA
};

class BlockReader
{
private:
    std::unique_ptr<node::BlockManager> m_blockman;
    std::unique_ptr<kernel::Notifications> m_notifications;

    fs::path m_data_dir;

    CChain m_validated_chain;
    int m_header_height{0};

public:
    explicit BlockReader(const fs::path& data_dir)
        : m_data_dir(data_dir)
    {
        class KernelNotifications : public kernel::Notifications
        {
        public:
            kernel::InterruptResult blockTip(SynchronizationState state, CBlockIndex& index, double verification_progress) override
            {
                return {};
            }
            void headerTip(SynchronizationState state, int64_t height, int64_t timestamp, bool presync) override {}
            void progress(const bilingual_str& title, int progress_percent, bool resume_possible) override {}
            void warningSet(kernel::Warning id, const bilingual_str& message) override {}
            void warningUnset(kernel::Warning id) override {}
            void flushError(const bilingual_str& message) override {}
            void fatalError(const bilingual_str& message) override {}
        };

        m_notifications = std::make_unique<KernelNotifications>();
    }

    bool Initialize()
    {
        LogPrintf("Initializing BlockReader...\n");
        LogPrintf("Data directory: %s\n", m_data_dir.utf8string());
        LogPrintf("Blocks directory: %s\n", (m_data_dir / "blocks").utf8string());

        static auto chainparams = CChainParams::SigNet(CChainParams::SigNetOptions{});
        static util::SignalInterrupt interrupt;

        const node::BlockManager::Options blockman_opts{
            .chainparams = *chainparams,
            .blocks_dir = m_data_dir / "blocks",
            .block_tree_dir = m_data_dir / "blocks" / "index",
            .notifications = *m_notifications,
        };
        m_blockman = std::make_unique<node::BlockManager>(interrupt, blockman_opts);

        LogPrintf("Loading Block index from %s...\n", (m_data_dir / "blocks" / "index").utf8string());

        std::vector<CBlockIndex*> validated_blocks;
        int max_header_height = 0;
        {
            LOCK(cs_main);

            if (!m_blockman->LoadBlockIndexDB({})) {
                LogPrintf("Failed to load block index\n");
                return false;
            }

            for (CBlockIndex* pindex : m_blockman->GetAllBlockIndices()) {
                max_header_height = std::max(max_header_height, pindex->nHeight);

                if (pindex->IsValid(BLOCK_VALID_SCRIPTS)) {
                    validated_blocks.push_back(pindex);
                }
            }
        }

        m_header_height = max_header_height;

        if (!validated_blocks.empty()) {
            std::sort(validated_blocks.begin(), validated_blocks.end(),
                      node::CBlockIndexWorkComparator());
            m_validated_chain.SetTip(*validated_blocks.back());
        }

        LogPrintf("Block index loaded successfully\n");
        LogPrintf("Header height: %d, Validated height: %d\n",
                  m_header_height, m_validated_chain.Height());

        return true;
    }

    void Refresh()
    {
        LogPrintf("Refreshing block index...\n");

        std::vector<CBlockIndex*> validated_blocks;
        int max_header_height = 0;

        {
            LOCK(cs_main);

            if (!m_blockman->LoadBlockIndexDB({})) {
                LogPrintf("Failed to load block index\n");
                return;
            }

            for (CBlockIndex* pindex : m_blockman->GetAllBlockIndices()) {
                max_header_height = std::max(max_header_height, pindex->nHeight);

                if (pindex->IsValid(BLOCK_VALID_SCRIPTS)) {
                    validated_blocks.push_back(pindex);
                }
            }
        }

        int previous_validated_height = m_validated_chain.Height();
        m_header_height = max_header_height;

        if (!validated_blocks.empty()) {
            std::sort(validated_blocks.begin(), validated_blocks.end(),
                      node::CBlockIndexWorkComparator());
            m_validated_chain.SetTip(*validated_blocks.back());
        }
        LogPrintf("Refresh complete: Header height: %d, Validated height: %d (+%d)\n",
                  m_header_height, m_validated_chain.Height(),
                  m_validated_chain.Height() - previous_validated_height);
    }

    IBDStatus GetIBDStatus() const
    {
        if (m_header_height == 0) return IBDStatus::NO_DATA;
        if (m_validated_chain.Height() == 0) return IBDStatus::IN_IBD;

        int blocks_behind = m_header_height - m_validated_chain.Height();
        return (blocks_behind > 144) ? IBDStatus::IN_IBD : IBDStatus::SYNCED;
    }

    CBlockIndex* GetBestValidatedBlock() const
    {
        return m_validated_chain.Tip();
    }

    std::optional<CBlock> GetBlockByHeight(int height) const
    {
        if (height < 0 || height >= m_validated_chain.Height()) {
            return std::nullopt;
        }

        CBlockIndex* pindex = m_validated_chain[height];
        return GetBlock(pindex->GetBlockHash());
    }

    std::optional<CBlock> GetBlock(const uint256& hash) const
    {
        CBlockIndex* pindex = nullptr;
        {
            LOCK(cs_main);
            pindex = m_blockman->LookupBlockIndex(hash);
        }

        if (!pindex) {
            LogPrintf("Block not found in index: %s\n", hash.ToString());
            return std::nullopt;
        }

        CBlock block;
        if (!m_blockman->ReadBlock(block, *pindex)) {
            LogPrintf("Failed to read block from disk: %s\n", hash.ToString());
            return std::nullopt;
        }

        return block;
    }
};

int main()
{
    LogInstance().m_print_to_console = true;
    LogInstance().m_log_timestamps = true;
    LogInstance().StartLogging();

    fs::path data_dir = fs::path("/Users/xyz/Library/Application Support/Bitcoin/signet");

    BlockReader block_reader(data_dir);


    if (!block_reader.Initialize()) {
        LogPrintf("Failed to initilialize Blockreader\n");
        return 1;
    }

    LogPrintf("BlockReader intialized\n");

    block_reader.Refresh();

    auto block_at_100 = block_reader.GetBlockByHeight(100);
    LogPrintf("Block 100 found: %s\n", block_at_100.value().ToString());

    return 0;
}
