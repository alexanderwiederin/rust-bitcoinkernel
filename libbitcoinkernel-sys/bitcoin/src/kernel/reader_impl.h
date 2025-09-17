// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_KERNEL_BLOCKREADER_READER_IMPL_H
#define BITCOIN_KERNEL_BLOCKREADER_READER_IMPL_H

#include <kernel/notifications_interface.h>
#include <kernel/chainparams.h>
#include <node/blockstorage.h>
#include <util/fs.h>
#include <util/signalinterrupt.h>
#include <memory>
namespace blockreader {

class BlockReader
{
private:
    std::unique_ptr<node::BlockManager> m_blockman;
    const util::SignalInterrupt& m_interrupt;
    std::unique_ptr<kernel::Notifications> m_notifications;
    CChain m_validated_chain;

    bool LoadBlockIndex();

public:
    struct Options {
        const CChainParams& chainparams;
        const fs::path blocks_dir;
        const fs::path data_dir;
    };

    BlockReader(const Options& options, util::SignalInterrupt& interrupt);
    BlockReader(const CChainParams& chain_params,
                const fs::path& data_dir,
                const fs::path& blocks_dir,
                util::SignalInterrupt& interrupt);

    ~BlockReader() = default;

    BlockReader(const BlockReader&) = delete;
    BlockReader& operator=(const BlockReader&) = delete;

    node::BlockManager& GetBlockManager() { return *m_blockman; }
    const node::BlockManager& GetBlockManager() const { return *m_blockman; }

    const CChain& GetValidatedChain() const { return m_validated_chain; }
};

} // namespace blockreader

#endif // BITCOIN_KERNEL_BLOCKREADER_READER_IMPL_H
