use bitcoin::{
    blockdata::constants::genesis_block, consensus::serialize, merkle_tree::MerkleBlock, Network,
};
use ckb_bitcoin_spv_verifier::{
    error::VerifyTxError,
    types::{core, packed, prelude::*},
};

use super::{prelude::BitcoinSpvStorage, Storage};

#[test]
fn generated_transaction_proof_rejects_a_tampered_header_mmr() {
    let directory = tempfile::tempdir().unwrap();
    let storage = Storage::new(directory.path()).unwrap();
    let block = genesis_block(Network::Bitcoin);
    let txid = block.txdata[0].txid();
    let mut client = storage.initialize_with(0, block.header).unwrap();

    // Add a linked header so the generated proof contains a sibling to tamper with.
    // This exercises storage/proof compatibility, not Bitcoin consensus validation.
    let mut next_header = block.header;
    next_header.prev_blockhash = block.block_hash();
    next_header.nonce = next_header.nonce.wrapping_add(1);
    storage.append_headers(vec![next_header]).unwrap();
    client.headers_mmr_root = storage.generate_headers_root(1).unwrap().unpack();
    client.tip_block_hash = next_header.block_hash().into();
    let client: packed::SpvClient = client.pack();
    let header_proof = storage.generate_headers_proof(1, vec![0]).unwrap();
    assert_eq!(header_proof.len(), 1);
    let merkle_block = MerkleBlock::from_block_with_predicate(&block, |id| *id == txid);
    let proof = packed::TransactionProof::new_builder()
        .tx_index(0u32.pack())
        .height(0u32.pack())
        .transaction_proof(core::Bytes::from(serialize(&merkle_block)).pack())
        .header_proof(header_proof.pack())
        .build();
    assert!(client
        .verify_transaction(txid.as_ref(), proof.as_reader(), 1)
        .is_ok());

    let mut forged_items = header_proof;
    forged_items[0].children_hash = core::Hash::hash(b"tampered sibling");
    let forged = proof.as_builder().header_proof(forged_items.pack()).build();
    assert_eq!(
        client
            .verify_transaction(txid.as_ref(), forged.as_reader(), 1)
            .map(|_| ())
            .map_err(|error| error as i8),
        Err(VerifyTxError::HeaderMmrProof as i8),
    );
}
