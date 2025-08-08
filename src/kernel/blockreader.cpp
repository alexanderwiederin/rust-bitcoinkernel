// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#include <chain.h>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <kernel/bitcoinkernel.h>
#include <kernel/blockreader.h>
#include <kernel/cs_main.h>
#include <kernel/reader_impl.h>
#include <logging.h>
#include <primitives/transaction.h>
#include <script/script.h>
#include <streams.h>

using namespace blockreader;

namespace {

BlockReader* cast_blockreader(kernel_blockreader_Reader* reader)
{
    assert(reader);
    return reinterpret_cast<BlockReader*>(reader);
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

const CBlock* cast_const_block_pointer(const kernel_BlockPointer* block_pointer)
{
    assert(block_pointer);
    return reinterpret_cast<const CBlock*>(block_pointer);
}

const CTransaction* cast_const_transaction(const kernel_Transaction* transaction)
{
    assert(transaction);
    return reinterpret_cast<const CTransaction*>(transaction);
}

const CTxIn* cast_const_transaction_input(const kernel_TransactionInput* input)
{
    assert(input);
    return reinterpret_cast<const CTxIn*>(input);
}

const CTxOut* cast_const_transaction_output(const kernel_TransactionOutput* output)
{
    assert(output);
    return reinterpret_cast<const CTxOut*>(output);
}

const COutPoint* cast_const_transaction_out_point(const kernel_TransactionOutPoint* out_point)
{
    assert(out_point);
    return reinterpret_cast<const COutPoint*>(out_point);
}

const CScript* cast_const_script_sig(const kernel_TransactionScriptSig* script_sig)
{
    assert(script_sig);
    return reinterpret_cast<const CScript*>(script_sig);
}

const CScriptWitness* cast_const_witness(const kernel_TransactionWitness* witness)
{
    assert(witness);
    return reinterpret_cast<const CScriptWitness*>(witness);
}

const CScript* cast_const_script_pubkey(const kernel_ScriptPubkey* script_pubkey)
{
    assert(script_pubkey);
    return reinterpret_cast<const CScript*>(script_pubkey);
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

const kernel_BlockIndex* kernel_blockreader_get_best_block_index(const kernel_blockreader_Reader* reader)
{
    auto br = cast_const_blockreader(reader);
    return reinterpret_cast<const kernel_BlockIndex*>(br->GetBestBlock());
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

uint32_t kernel_block_index_get_version(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    return bi->nVersion;
}

const kernel_Hash* kernel_block_index_get_merkle_root(const kernel_BlockIndex* block_index)
{
    auto* bi = cast_const_block_index(block_index);

    auto merkle_root = bi->hashMerkleRoot;
    auto hash = new kernel_Hash{};

    std::memcpy(hash->hash, merkle_root.begin(), sizeof(merkle_root));
    return hash;
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

bool kernel_block_index_has_block_data(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_HAVE_DATA;
}

bool kernel_block_index_has_undo_data(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_HAVE_UNDO;
}

bool kernel_block_index_has_valid_transactions(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_TRANSACTIONS);
}

bool kernel_block_index_has_valid_chain(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_CHAIN);
}

bool kernel_block_index_has_valid_scripts(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->IsValid(BLOCK_VALID_SCRIPTS);
}

bool kernel_block_index_is_failed(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_FAILED_VALID;
}

bool kernel_block_index_has_witness(const kernel_BlockIndex* block_index)
{
    LOCK(cs_main);
    auto* bi = cast_const_block_index(block_index);

    return bi->nStatus & BLOCK_OPT_WITNESS;
}

const kernel_BlockPointer* kernel_blockreader_get_block_by_index(const kernel_blockreader_Reader* reader, const kernel_BlockIndex* block_index_)
{
    auto br = cast_const_blockreader(reader);
    const CBlockIndex* block_index{cast_const_block_index(block_index_)};

    auto block_opt = br->GetBlock(block_index);
    if (!block_opt.has_value()) {
        LogError("Failed to read block.");
        return nullptr;
    }

    return reinterpret_cast<const kernel_BlockPointer*>(block_opt.value());
}

size_t kernel_block_pointer_get_transaction_count(const kernel_BlockPointer* block_pointer)
{
    const auto* block = cast_const_block_pointer(block_pointer);
    return block->vtx.size();
}

const kernel_Transaction* kernel_block_pointer_get_transaction(const kernel_BlockPointer* block_pointer, size_t index)
{
    const auto* block = cast_const_block_pointer(block_pointer);
    if (index >= block->vtx.size()) {
        return nullptr;
    }
    return reinterpret_cast<const kernel_Transaction*>(block->vtx[index].get());
}

const kernel_Hash* kernel_transaction_get_hash(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    auto tx_id = transaction->GetHash();

    auto hash = new kernel_Hash{};
    std::memcpy(hash->hash, tx_id.begin(), sizeof(tx_id));
    return hash;
}

bool kernel_transaction_is_null(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->IsNull();
}

const kernel_Hash* kernel_transaction_get_witness_hash(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    auto witness_hash = transaction->GetWitnessHash();

    auto hash = new kernel_Hash{};
    std::memcpy(hash->hash, witness_hash.begin(), sizeof(witness_hash));
    return hash;
}

int64_t kernel_transaction_get_value_out(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->GetValueOut();
}

size_t kernel_transaction_get_total_size(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->GetTotalSize();
}

bool kernel_transaction_is_coinbase(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->IsCoinBase();
}

bool kernel_transaction_has_witness(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->HasWitness();
}

size_t kernel_transaction_get_input_count(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);
    return transaction->vin.size();
}

const kernel_TransactionInput* kernel_transaction_get_input(const kernel_Transaction* transaction_, size_t index)
{
    const auto* transaction = cast_const_transaction(transaction_);
    if (index >= transaction->vin.size()) {
        return nullptr;
    }
    return reinterpret_cast<const kernel_TransactionInput*>(&transaction->vin[index]);
}

const kernel_TransactionOutPoint* kernel_transaction_input_get_out_point(const kernel_TransactionInput* input_)
{
    const auto* input = cast_const_transaction_input(input_);
    return reinterpret_cast<const kernel_TransactionOutPoint*>(&input->prevout);
}

const kernel_Hash* kernel_transaction_out_point_get_hash(const kernel_TransactionOutPoint* out_point_)
{
    const auto* out_point = cast_const_transaction_out_point(out_point_);

    auto* hash = new kernel_Hash{};
    std::memcpy(hash->hash, out_point->hash.begin(), sizeof(hash->hash));

    return hash;
}

uint32_t kernel_transaction_out_point_get_index(const kernel_TransactionOutPoint* out_point_)
{
    const auto* out_point = cast_const_transaction_out_point(out_point_);

    return out_point->n;
}

const kernel_TransactionScriptSig* kernel_transaction_input_get_script_sig(const kernel_TransactionInput* input_)
{
    const auto* input = cast_const_transaction_input(input_);

    return reinterpret_cast<const kernel_TransactionScriptSig*>(&input->scriptSig);
}

bool kernel_script_sig_is_push_only(const kernel_TransactionScriptSig* script_sig_)
{
    const auto* script_sig = cast_const_script_sig(script_sig_);

    return script_sig->IsPushOnly();
}

bool kernel_script_sig_is_empty(const kernel_TransactionScriptSig* script_sig_)
{
    const auto* script_sig = cast_const_script_sig(script_sig_);

    return script_sig->empty();
}

uint32_t kernel_transaction_input_get_n_sequence(const kernel_TransactionInput* input_)
{
    const auto* input = cast_const_transaction_input(input_);

    return input->nSequence;
}

const kernel_TransactionWitness* kernel_transaction_input_get_witness(const kernel_TransactionInput* input_)
{
    const auto* input = cast_const_transaction_input(input_);

    return reinterpret_cast<const kernel_TransactionWitness*>(&input->scriptWitness);
}

size_t kernel_witness_get_stack_size(const kernel_TransactionWitness* witness_)
{
    const auto* witness = cast_const_witness(witness_);

    return witness->stack.size();
}

kernel_ByteArray* kernel_witness_get_stack_item(const kernel_TransactionWitness* witness_, size_t index)
{
    const auto* witness = cast_const_witness(witness_);

    if (index >= witness->stack.size()) {
        return nullptr;
    }

    const auto& stack_item = witness->stack.at(index);

    auto byte_array = new kernel_ByteArray();
    byte_array->size = stack_item.size();
    byte_array->data = new unsigned char[byte_array->size];

    std::memcpy(byte_array->data, stack_item.data(), byte_array->size);

    return byte_array;
}

bool kernel_witness_is_null(const kernel_TransactionWitness* witness_)
{
    const auto* witness = cast_const_witness(witness_);

    return witness->IsNull();
}

size_t kernel_transaction_get_output_count(const kernel_Transaction* transaction_)
{
    const auto* transaction = cast_const_transaction(transaction_);

    return transaction->vout.size();
}

const kernel_TransactionOutput* kernel_transaction_get_output(const kernel_Transaction* transaction_, size_t index)
{
    const auto* transaction = cast_const_transaction(transaction_);

    if (index >= transaction->vout.size()) {
        return nullptr;
    }

    return reinterpret_cast<const kernel_TransactionOutput*>(&transaction->vout[index]);
}

const kernel_ScriptPubkey* kernel_transaction_output_get_script_pubkey(const kernel_TransactionOutput* output_)
{
    const auto* output = cast_const_transaction_output(output_);

    return reinterpret_cast<const kernel_ScriptPubkey*>(&output->scriptPubKey);
}

size_t kernel_script_pubkey_get_size(const kernel_ScriptPubkey* script_pubkey_)
{
    const auto* script_pubkey = cast_const_script_pubkey(script_pubkey_);
    return script_pubkey->size();
}

const unsigned char* kernel_script_pubkey_get_data(const kernel_ScriptPubkey* script_pubkey_)
{
    const auto* script_pubkey = cast_const_script_pubkey(script_pubkey_);
    return script_pubkey->data();
}

const kernel_BlockUndo* kernel_blockreader_get_undo_data(const kernel_blockreader_Reader* reader_, const kernel_BlockIndex* block_index_)
{
    const auto* reader = cast_const_blockreader(reader_);
    const auto block_index{cast_const_block_index(block_index_)};

    auto block_undo_opt = reader->GetUndoData(block_index);
    if (!block_undo_opt.has_value()) {
        LogError("Failed to get undo data.");
        return nullptr;
    }

    return reinterpret_cast<const kernel_BlockUndo*>(block_undo_opt.value());
}

const unsigned char* kernel_script_sig_get_data(const kernel_TransactionScriptSig* script_sig_)
{
    const auto* script_sig = cast_const_script_sig(script_sig_);
    return script_sig->data();
}

size_t kernel_script_sig_get_size(const kernel_TransactionScriptSig* script_sig_)
{
    const auto* script_sig = cast_const_script_sig(script_sig_);
    return script_sig->size();
}

const kernel_BlockIndex* kernel_blockreader_get_block_index_by_height(const kernel_blockreader_Reader* reader_, int height)
{
    auto reader{cast_const_blockreader(reader_)};
    return reinterpret_cast<const kernel_BlockIndex*>(reader->GetBlockByHeight(height));
}

bool kernel_block_index_is_on_best_chain(const kernel_blockreader_Reader* reader_, const kernel_BlockIndex* block_index_)
{
    auto reader{cast_const_blockreader(reader_)};
    auto block_index = cast_const_block_index(block_index_);
    return reader->IsOnBestChain(block_index);
}

} // extern "C"
