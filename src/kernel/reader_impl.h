// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_KERNEL_BLOCKREADER_READER_IMPL_H
#define BITCOIN_KERNEL_BLOCKREADER_READER_IMPL_H

#include <chain.h>
#include <cstdint>
#include <kernel/chainparams.h>
#include <kernel/notifications_interface.h>
#include <memory>
#include <node/blockstorage.h>
#include <optional>
#include <uint256.h>
#include <undo.h>
#include <util/fs.h>
#include <util/signalinterrupt.h>
#include <validation.h>

namespace blockreader {

enum class IBDStatus {
    NO_DATA,
    IN_IBD,
    SYNCED
};

class BlockReader
{
private:
    std::unique_ptr<node::BlockManager> m_blockman;
    std::unique_ptr<kernel::Notifications> m_notifications;
    std::unique_ptr<util::SignalInterrupt> m_interrupt;
    std::unique_ptr<const CChainParams> m_chainparams;

    fs::path m_data_dir;
    CChain m_validated_chain;
    int m_header_height{0};

    class KernelNotifications : public kernel::Notifications
    {
    public:
        kernel::InterruptResult blockTip(SynchronizationState state, CBlockIndex& index, double verification_progress) override { return {}; }
        void headerTip(SynchronizationState state, int64_t height, int64_t timestamp, bool presync) override {}
        void progress(const bilingual_str& title, int progress_percent, bool resume_possible) override {}
        void warningSet(kernel::Warning id, const bilingual_str& message) override {}
        void warningUnset(kernel::Warning id) override {}
        void flushError(const bilingual_str& message) override {}
        void fatalError(const bilingual_str& message) override {}
    };

public:
    explicit BlockReader(const CChainParams& chain_params, const fs::path& data_dir);
    ~BlockReader() = default;

    BlockReader(const BlockReader&) = delete;
    BlockReader& operator=(const BlockReader&) = delete;

    bool Initialize();

    bool Refresh();

    IBDStatus GetIBDStatus() const;

    const CBlockIndex* GetBestBlock() const;

    const CBlockIndex* GetBlockByHeight(int height) const;

    std::optional<const CBlock*> GetBlock(const CBlockIndex* block_index) const;

    std::optional<const CBlockUndo*> GetUndoData(const CBlockIndex* block_index) const;

    bool IsOnBestChain(const CBlockIndex* block_index) const;

private:
    bool LoadBlockIndex();
};
} // namespace blockreader
#endif
