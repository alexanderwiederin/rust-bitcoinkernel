// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include "chain.h"
#include "kernel/bitcoinkernel.h"
#include "kernel/cs_main.h"
#include "logging.h"
#include "streams.h"
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <kernel/blockreader/blockreader.h>
#include <kernel/blockreader/reader_impl.h>

using namespace blockreader;

namespace {

BlockReader* cast_blockreader(kernel_blockreader_Reader* reader)
{
    assert(reader);
    return reinterpret_cast<BlockReader*>(reader);
}

CBlock* cast_block(kernel_Block* block)
{
    assert(block);
    return reinterpret_cast<CBlock*>(block);
}

const BlockReader* cast_const_blockreader(const kernel_blockreader_Reader* reader)
{
    assert(reader);
    return reinterpret_cast<const BlockReader*>(reader);
}

const CBlockIndex* cast_const_block_index(const kernel_BlockIndex* block_index)
{
    assert(block_index);
    return reinterpret_cast<const CBlockIndex*>(block_index);
}

kernel_blockreader_IBDStatus cast_ibd_status(IBDStatus status)
{
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
    size_t data_dir_len)
{
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

void kernel_blockreader_refresh(kernel_blockreader_Reader* reader)
{
    auto br = cast_blockreader(reader);
    br->Refresh();
}

void kernel_blockreader_destroy(kernel_blockreader_Reader* reader)
{
    if (reader) {
        delete cast_blockreader(reader);
    }
}

kernel_blockreader_IBDStatus kernel_blockreader_get_ibd_status(const kernel_blockreader_Reader* reader)
{
    auto br = cast_const_blockreader(reader);
    return cast_ibd_status(br->GetIBDStatus());
}

kernel_BlockIndex* kernel_blockreader_get_best_validated_block(const kernel_blockreader_Reader* reader)
{
    auto br = cast_const_blockreader(reader);
    return reinterpret_cast<kernel_BlockIndex*>(br->GetBestValidatedBlock());
}

kernel_Block* kernel_blockreader_get_block_by_height(const kernel_blockreader_Reader* reader, int32_t height)
{
    auto br = cast_const_blockreader(reader);
    auto block_opt = br->GetBlockByHeight(height);

    if (!block_opt) return nullptr;

    auto block = new CBlock{*block_opt};
    return reinterpret_cast<kernel_Block*>(block);
}

kernel_BlockHash* kernel_blockreader_block_get_hash(kernel_Block* block)
{
    auto cblock = cast_block(block);
    auto hash = cblock->GetHash();

    auto block_hash = new kernel_BlockHash{};
    std::memcpy(block_hash->hash, hash.begin(), sizeof(hash));
    return block_hash;
}

void kernel_blockreader_block_destroy(kernel_Block* block)
{
    delete cast_block(block);
}

kernel_BlockIndex* kernel_blockreader_get_block_index_by_height(
    const kernel_blockreader_Reader* reader,
    int32_t height)
{
    auto br = cast_const_blockreader(reader);
    return reinterpret_cast<kernel_BlockIndex*>(br->GetBlockIndexByHeight(height));
}

kernel_BlockHash* kernel_blockreader_get_genesis_hash(const kernel_blockreader_Reader* reader)
{
    auto br = cast_const_blockreader(reader);

    uint256 genesis_hash = br->GetGenesisHash();

    auto hash = new kernel_BlockHash{};

    std::memcpy(hash->hash, genesis_hash.begin(), sizeof(genesis_hash));

    return hash;
}

bool kernel_blockreader_is_block_in_active_chain(
    const kernel_blockreader_Reader* reader,
    const kernel_BlockIndex* block_index)
{
    auto br = cast_const_blockreader(reader);
    auto bi = reinterpret_cast<const CBlockIndex*>(block_index);

    auto cblock_at_height = br->GetBlockIndexByHeight(bi->nHeight);
    return cblock_at_height->GetBlockHash() == bi->GetBlockHash();
}

kernel_ByteArray* kernel_blockreader_get_headers_raw(
    const kernel_blockreader_Reader* reader,
    int32_t start_height,
    size_t count)
{
    try {
        if (count == 0) {
            return nullptr;
        }

        auto br = cast_const_blockreader(reader);

        std::vector<unsigned char> header_data;
        header_data.reserve(count * 80);

        DataStream stream;
        size_t retrieved = 0;

        for (size_t i = 0; i < count; i++) {
            int32_t height = start_height + static_cast<int32_t>(i);

            auto block_index = br->GetBlockIndexByHeight(height);
            if (!block_index) {
                break;
            }

            const auto* cblock_index = reinterpret_cast<const CBlockIndex*>(block_index);
            const CBlockHeader& header = cblock_index->GetBlockHeader();

            stream = DataStream{};
            stream << header;

            if (stream.size() != 80) {
                LogError("Header size error at height %d", height);
                break;
            }

            std::vector<unsigned char> stream_data(stream.size());
            std::memcpy(stream_data.data(), stream.data(), stream.size());
            header_data.insert(header_data.end(), stream_data.begin(), stream_data.end());
            retrieved++;
        }

        if (retrieved == 0) return nullptr;

        auto batch = new kernel_ByteArray{};
        batch->size = header_data.size();
        batch->data = new unsigned char[batch->size];
        std::memcpy(batch->data, header_data.data(), batch->size);

        return batch;
    } catch (const std::exception& e) {
        LogError("Failed to get headers raw: %s", e.what());
        return nullptr;
    }
}

kernel_ByteArray* kernel_block_index_get_raw_header(
    const kernel_BlockIndex* block_index)
{
    try {
        const auto* cblock_index = cast_const_block_index(block_index);
        const CBlockHeader& header = cblock_index->GetBlockHeader();

        auto byte_array = new kernel_ByteArray{};
        byte_array->size = 80;
        byte_array->data = new unsigned char[80];

        DataStream stream = DataStream{};
        stream << header;
        std::memcpy(byte_array->data, stream.data(), 80);

        return byte_array;
    } catch (const std::exception& e) {
        LogError("Failed to get raw header: %s", e.what());
        return nullptr;
    }
}

uint32_t kernel_block_index_get_timestamp(const kernel_BlockIndex* block_index)
{
    const auto* cblock_index = cast_const_block_index(block_index);
    return cblock_index->GetBlockHeader().nTime;
}

uint32_t kernel_block_index_get_transaction_count(const kernel_BlockIndex* block_index)
{
    const auto* cblock_index = cast_const_block_index(block_index);
    return cblock_index->nTx;
}

kernel_BlockIndex* kernel_blockreader_get_block_index_by_hash(
    const kernel_blockreader_Reader* reader,
    const kernel_BlockHash* hash)
{
    auto br = cast_const_blockreader(reader);
    uint256 hash_uint256;
    std::memcpy(hash_uint256.begin(), hash->hash, 32);
    return reinterpret_cast<kernel_BlockIndex*>(br->GetBlockIndex(hash_uint256));
}

kernel_Block* kernel_blockreader_get_block_by_hash(
    const kernel_blockreader_Reader* reader,
    const kernel_BlockHash* hash)
{
    try {
        auto br = cast_const_blockreader(reader);

        uint256 block_hash;
        std::memcpy(block_hash.begin(), hash->hash, 32);

        auto block_opt = br->GetBlock(block_hash);
        if (!block_opt) return nullptr;

        auto block = new CBlock{*block_opt};
        return reinterpret_cast<kernel_Block*>(block);
    } catch (const std::exception& e) {
        LogError("Failed to get block by hash: %s", e.what());
        return nullptr;
    }
}

kernel_BlockHash* kernel_block_index_get_previous_block_hash(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    CBlockIndex* prev_index = bi->pprev;
    if (!prev_index) return nullptr;

    auto prev_block_hash = prev_index->GetBlockHash();

    auto block_hash = new kernel_BlockHash{};
    std::memcpy(block_hash->hash, prev_block_hash.begin(), sizeof(prev_block_hash));
    return block_hash;
}

uint32_t kernel_block_index_get_version(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    return bi->nVersion;
}

kernel_BlockHash* kernel_block_index_get_merkle_root(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    auto merkle_root = bi->hashMerkleRoot;
    auto block_hash = new kernel_BlockHash{};

    std::memcpy(block_hash->hash, merkle_root.begin(), sizeof(merkle_root));
    return block_hash;
}

uint32_t kernel_block_index_get_bits(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    return bi->nBits;
}

uint32_t kernel_block_index_get_nonce(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    return bi->nNonce;
}

uint32_t kernel_block_index_get_median_time_past(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    return bi->GetMedianTimePast();
}

bool kernel_block_index_has_block_data(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_HAVE_DATA;
}

bool kernel_block_index_has_undo_data(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_HAVE_UNDO;
}

bool kernel_block_index_has_valid_transactions(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_TRANSACTIONS);
}

bool kernel_block_index_has_valid_chain(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_CHAIN);
}

bool kernel_block_index_has_valid_scripts(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_SCRIPTS);
}

bool kernel_block_index_is_failed(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_FAILED_VALID;
}

bool kernel_block_index_has_witness(const kernel_BlockIndex *block_index) {
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_OPT_WITNESS;
}

} // extern "C"
