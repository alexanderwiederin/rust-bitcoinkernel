// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include "kernel/bitcoinkernel.h"
#include "kernel/context.h"
#include "logging.h"
#include <cstdint>
#include <kernel/blockreader/blockreader.h>
#include <kernel/blockreader/reader_impl.h>

using namespace blockreader;

namespace {

    BlockReader* cast_blockreader(kernel_blockreader_Reader* reader) {
        assert(reader);
        return reinterpret_cast<BlockReader*>(reader);
    }

    const BlockReader* cast_const_blockreader(const kernel_blockreader_Reader* reader) {
        assert(reader);
        return reinterpret_cast<const BlockReader*>(reader);
    }

    kernel_blockreader_IBDStatus cast_ibd_status(IBDStatus status) {
        switch (status) {
            case IBDStatus::NO_DATA:
                return kernel_blockreader_IBD_STATUS_NO_DATA;
            case IBDStatus::IN_IBD:
                return kernel_blockreader_IBD_STATUS_IN_IBD;
            case IBDStatus::SYNCED:
                return kernel_blockreader_IBD_STATUS_SYNCED;
        }
        assert(false);
    }

} // namespace

extern "C" {
    kernel_blockreader_Reader* kernel_blockreader_create(
            const kernel_ChainParameters* chain_params,
            const char* data_dir,
            size_t data_dir_len) {

        try {
            const auto* cchainparams = reinterpret_cast<const CChainParams*>(chain_params);

            fs::path abs_data_dir{fs::absolute(fs::PathFromString({data_dir, data_dir_len}))};
            auto reader = new BlockReader{*cchainparams, abs_data_dir};

            if (!reader->Initialize()) {
                delete reader;
                return nullptr;
            }

            return reinterpret_cast<kernel_blockreader_Reader*>(reader);
        } catch (const std::exception& e) {
            LogError("Failed to create BlockReader: %s", e.what());
            return nullptr;
        }
    }

    void kernel_blockreader_refresh(kernel_blockreader_Reader* reader) {
        auto br = cast_blockreader(reader);
        br->Refresh();
    }

    void kernel_blockreader_destroy(kernel_blockreader_Reader* reader) {
        if (reader) {
            delete cast_blockreader(reader);
        }
    }

    kernel_blockreader_IBDStatus kernel_blockreader_get_ibd_status(const kernel_blockreader_Reader* reader) {
        auto br = cast_const_blockreader(reader);
        return cast_ibd_status(br->GetIBDStatus());
    }

    kernel_BlockIndex* kernel_blockreader_get_best_validated_block(const kernel_blockreader_Reader* reader) {
        auto br = cast_const_blockreader(reader);
        return reinterpret_cast<kernel_BlockIndex*>(br->GetBestValidatedBlock());
    }

    kernel_Block* kernel_blockreader_get_block_by_height(const kernel_blockreader_Reader *reader, int32_t height) {
        auto br = cast_const_blockreader(reader);
        auto block_opt = br->GetBlockByHeight(height);

        if (!block_opt) return nullptr;

        auto block = new CBlock{*block_opt};
        return reinterpret_cast<kernel_Block*>(block);
    }

    kernel_BlockHash* kernel_blockreader_block_get_hash(kernel_Block* block) {
        auto cblock = reinterpret_cast<CBlock*>(block);
        auto hash = cblock->GetHash();

        auto block_hash = new kernel_BlockHash{};
        std::memcpy(block_hash->hash, hash.begin(), sizeof(hash));
        return block_hash;
    }

    void kernel_blockreader_block_destroy(kernel_Block* block) {
        if (block) {
            delete reinterpret_cast<CBlock*>(block);
        }
    }

    kernel_BlockIndex* kernel_blockreader_get_block_index_by_height(
            const kernel_blockreader_Reader* reader,
            int32_t height) {

        auto br = cast_const_blockreader(reader);
        return reinterpret_cast<kernel_BlockIndex*>(br->GetBlockIndexByHeight(height));
    }

} // extern "C"
