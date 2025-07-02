// Copyright (c) 2022-present The Bitcoin Core developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.

#ifndef BITCOIN_KERNEL_BLOCKREADER_BLOCKREADER_H
#define BITCOIN_KERNEL_BLOCKREADER_BLOCKREADER_H

#include <kernel/bitcoinkernel.h>

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

typedef struct kernel_blockreader_Reader kernel_blockreader_Reader;

typedef enum {
    kernel_blockreader_IBD_STATUS_NO_DATA,
    kernel_blockreader_IBD_STATUS_IN_IBD,
    kernel_blockreader_IBD_STATUS_SYNCED
} kernel_blockreader_IBDStatus;

BITCOINKERNEL_API kernel_blockreader_Reader* BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_create(
        const kernel_ChainParameters* chain_params,
        const char* data_dir,
        size_t data_dir_len
) BITCOINKERNEL_ARG_NONNULL(1, 2);

BITCOINKERNEL_API void kernel_blockreader_refresh(
        kernel_blockreader_Reader* reader
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API void kernel_blockreader_destroy(kernel_blockreader_Reader* reader);

BITCOINKERNEL_API kernel_blockreader_IBDStatus kernel_blockreader_get_ibd_status(
        const kernel_blockreader_Reader* reader
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API int32_t BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_header_height(
        const kernel_blockreader_Reader* reader
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API int32_t BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_validated_height(
        const kernel_blockreader_Reader* reader
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API kernel_BlockIndex* BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_best_validated_block(
        const kernel_blockreader_Reader* reader
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API kernel_Block* BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_block_by_height(
        const kernel_blockreader_Reader* reader,
        int32_t height
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API kernel_Block* BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_block_by_hash(
        const kernel_blockreader_Reader* reader,
        const kernel_BlockHash* block_hash
) BITCOINKERNEL_ARG_NONNULL(1, 2);

BITCOINKERNEL_API kernel_BlockIndex* BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_get_block_index_by_height(
        const kernel_blockreader_Reader* reader,
        int32_t height
) BITCOINKERNEL_ARG_NONNULL(1);

BITCOINKERNEL_API bool BITCOINKERNEL_WARN_UNUSED_RESULT kernel_blockreader_block_index_is_on_main_chain(
        const kernel_blockreader_Reader* reader,
        const kernel_BlockIndex* index
) BITCOINKERNEL_ARG_NONNULL(1, 2);

#ifdef __cplusplus
}
#endif

#endif
