
#include <chain.h>
#include <cstddef>
#include <exception>
#include <kernel/chainparams.h>
#include <logging.h>
#include <memory>
#include <node/blockstorage.h>
#include <optional>
#include <reader_impl.h>
#include <uint256.h>
#include <undo.h>
#include <util/signalinterrupt.h>

namespace blockreader {

BlockReader::BlockReader(const CChainParams& chain_params, const fs::path& data_dir)
    : m_notifications(std::make_unique<KernelNotifications>()), m_interrupt(std::make_unique<util::SignalInterrupt>()), m_chainparams(std::make_unique<CChainParams>(chain_params)), m_data_dir(data_dir)
{
}

bool BlockReader::Initialize()
{
    LogPrintf("Initalizing BlockReader...\n");
    LogPrintf("Data directory: %s\n", m_data_dir.utf8string());
    LogPrintf("Blocks directory: %s\n", (m_data_dir / "blocks").utf8string());

    const node::BlockManager::Options blockman_opts{
        .chainparams = *m_chainparams,
        .blocks_dir = m_data_dir / "blocks",
        .block_tree_dir = m_data_dir / "blocks" / "index",
        .notifications = *m_notifications,
        .read_only = true,
    };

    try {
        m_blockman = std::make_unique<node::BlockManager>(*m_interrupt, blockman_opts);
    } catch (const std::exception& e) {
        LogError("Failed to create BlockManager: %s", e.what());
        return false;
    }

    LogPrintf("Loading Block index from %s...\n", (m_data_dir / "blocks" / "index").utf8string());

    if (!LoadBlockIndex()) {
        LogPrintf("Failed to load block index\n");
        return false;
    }

    LogPrintf("Block index loaded successfully\n");
    LogPrintf("Header height: %d, Validated height: %d\n",
              m_header_height, m_validated_chain.Height());

    return true;
}

bool BlockReader::LoadBlockIndex()
{
    std::vector<CBlockIndex*> validated_blocks;
    int max_header_height = 0;

    {
        LOCK(cs_main);

        if (!m_blockman->LoadBlockIndexDB({})) {
            LogPrintf("Failed to load blokc index database\n");
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

    return true;
}

void BlockReader::Refresh()
{
    LogPrintf("Refreshing block index...\n");

    int previous_validated_height = m_validated_chain.Height();

    if (!LoadBlockIndex()) {
        LogPrintf("Failed to refresh block index\n");
        return;
    }

    LogPrintf("Refresh complete: Header height: %d, Validated hieght: %d (+%d)\n",
              m_header_height, m_validated_chain.Height(),
              m_validated_chain.Height() - previous_validated_height);
}

IBDStatus BlockReader::GetIBDStatus() const
{
    if (m_header_height == 0) return IBDStatus::NO_DATA;
    if (m_validated_chain.Height() == 0) return IBDStatus::IN_IBD;

    int blocks_behind = m_header_height - m_validated_chain.Height();
    return (blocks_behind > 144) ? IBDStatus::IN_IBD : IBDStatus::SYNCED;
}

CBlockIndex* BlockReader::GetBestBlock() const
{
    return m_validated_chain.Tip();
}

std::optional<CBlock*> BlockReader::GetBlockByHeight(int height) const
{
    if (height < 0 || height > m_validated_chain.Height()) {
        LogDebug(BCLog::BLOCKSTORAGE, "Block hieght %d is out of range [0, %d]\n",
                 height, m_validated_chain.Height());
        return std::nullopt;
    }

    const CBlockIndex* pindex = m_validated_chain[height];
    if (!pindex) {
        LogDebug(BCLog::BLOCKSTORAGE, "Block at height %d is null\n", height);
        return std::nullopt;
    }

    return GetBlockByIndex(pindex);
}

std::optional<CBlock*> BlockReader::GetBlockByIndex(const CBlockIndex* block_index) const
{
    auto block = new CBlock{};
    if (!m_blockman->ReadBlock(*block, *block_index)) {
        LogPrintf("Failed to read block from disk: %s\n", block_index->GetBlockHash().ToString());
        delete block;
        return std::nullopt;
    }
    return block;
}

CBlockIndex* BlockReader::GetBlockIndexByHash(const uint256& hash) const
{
    LOCK(cs_main);
    return m_blockman->LookupBlockIndex(hash);
}

CBlockIndex* BlockReader::GetBlockIndexByHeight(const int height) const
{
    if (height < 0 || height > m_validated_chain.Height()) {
        return nullptr;
    }
    return m_validated_chain[height];
}

uint256 BlockReader::GetGenesisHash() const
{
    return m_validated_chain.Genesis()->GetBlockHash();
}

CBlockUndo* BlockReader::GetUndoData(const CBlockIndex* block_index) const
{
    if (block_index->nHeight < 1) {
        LogDebug(BCLog::KERNEL, "The genesis block does not have undo data.");
        return nullptr;
    }

    auto block_undo{new CBlockUndo{}};
    if (!m_blockman->ReadBlockUndo(*block_undo, *block_index)) {
        LogError("Failed to read block undo data.");
        return nullptr;
    }

    return block_undo;
}
} // namespace blockreader
