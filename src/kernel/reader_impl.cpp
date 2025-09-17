// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <kernel/reader_impl.h>
#include <chain.h>
#include <kernel/chainparams.h>
#include <kernel/cs_main.h>
#include <kernel/notifications_interface.h>
#include <node/blockstorage.h>
#include <validation.h>
#include <algorithm>
#include <logging.h>
#include <memory>
#include <sync.h>
#include <util/signalinterrupt.h>
#include <vector>

namespace blockreader {
class ReadOnlyNotifications : public kernel::Notifications
{
public:
    kernel::InterruptResult blockTip(SynchronizationState state, const CBlockIndex& index, double verification_progress) override { return {}; }
    void headerTip(SynchronizationState state, int64_t height, int64_t timestamp, bool presync) override {}
    void progress(const bilingual_str& title, int progress_percent, bool resume_possible) override {}
    void warningSet(kernel::Warning id, const bilingual_str& message) override {}
    void warningUnset(kernel::Warning id) override {}
    void flushError(const bilingual_str& message) override {}
    void fatalError(const bilingual_str& message) override {}
};

BlockReader::BlockReader(const Options& options, util::SignalInterrupt& interrupt)
    : BlockReader(options.chainparams, options.data_dir, options.blocks_dir, interrupt) {}

BlockReader::BlockReader(const CChainParams& chain_params,
                         const fs::path& data_dir,
                         const fs::path& blocks_dir,
                         util::SignalInterrupt& interrupt)
    : m_interrupt(interrupt), m_notifications(std::make_unique<ReadOnlyNotifications>())
{
    auto notifications = std::make_unique<ReadOnlyNotifications>();

    node::BlockManager::Options blockman_options{
        .chainparams = chain_params,
        .blocks_dir = blocks_dir,
        .notifications = *notifications,
        .block_tree_dir = data_dir / "blocks" / "index",
        .read_only = true};

    m_blockman = std::make_unique<node::BlockManager>(m_interrupt, blockman_options);

    if (!LoadBlockIndex()) {
        LogError("BlockReader: Failed to load block index");
        throw std::runtime_error("Failed to load block index");
    }
}

bool BlockReader::LoadBlockIndex()
{
    std::vector<CBlockIndex*> validated_blocks;

    {
        LOCK(cs_main);
        if (!m_blockman->LoadBlockIndexDB({})) {
            LogError("Failed to load block index database");
            return false;
        }

        for (CBlockIndex* pindex : m_blockman->GetAllBlockIndices()) {
            if (pindex->IsValid(BLOCK_VALID_SCRIPTS)) {
                validated_blocks.push_back(pindex);
            }
        }
    }

    if (!validated_blocks.empty()) {
        std::sort(validated_blocks.begin(), validated_blocks.end(), node::CBlockIndexWorkComparator());
        m_validated_chain.SetTip(*validated_blocks.back());
    }

    return true;
}


} // namespace blockreader
