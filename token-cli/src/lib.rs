use std::error::Error;

use solana_sdk::{instruction::AccountMeta, sysvar::SysvarId};
macro_rules! get_token_account {
    ($src:ident,$id:ident) => {
        solana_program::pubkey::Pubkey::find_program_address(&[$src.as_ref()], &$id).0
    };
}
pub mod error;
pub mod logs;
pub fn airdrop(
    program_id: solana_program::pubkey::Pubkey,
    dest: &solana_program::pubkey::Pubkey,
    amount: u64,
) -> solana_program::instruction::Instruction {
    let mut data = Vec::with_capacity(80);
    data.push(2);
    data.extend_from_slice(&amount.to_be_bytes());
    solana_program::instruction::Instruction {
        program_id: program_id.clone(),
        accounts: vec![
            AccountMeta::new(dest.clone(), true),
            AccountMeta::new(get_token_account!(dest, program_id), false),
            AccountMeta::new_readonly(program_id, false),
        ],
        data: data,
    }
}
pub fn init_account(
    program_id: solana_program::pubkey::Pubkey,
    authority: &solana_program::pubkey::Pubkey,
) -> solana_program::instruction::Instruction {
    // let (account, _) =
    let (account, bump) =
        solana_program::pubkey::Pubkey::find_program_address(&[authority.as_ref()], &program_id);
    println!("pda {}", account.to_string());
    solana_program::instruction::Instruction::new_with_bytes(
        program_id,
        &[0, bump],
        vec![
            AccountMeta::new(authority.clone(), true),
            AccountMeta::new(account, false),
            AccountMeta::new_readonly(solana_sdk::rent::Rent::id(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    )
}
pub fn get_balance(
    program_id: solana_program::pubkey::Pubkey,
    target: &solana_program::pubkey::Pubkey,
    c: &solana_client::rpc_client::RpcClient,
) -> Result<u64, Box<dyn Error>> {
    let account = get_token_account!(target, program_id);
    let data = c.get_account_data(&account)?;
    let dd = data.as_slice();
    let mut d: [u8; 8] = [0u8; 8];
    d.copy_from_slice(&dd[0..8]);
    Ok(u64::from_be_bytes(d))
}
pub fn transfer(
    program_id: solana_program::pubkey::Pubkey,
    from: &solana_program::pubkey::Pubkey,
    to: &solana_program::pubkey::Pubkey,
    amount: u64,
) -> solana_program::instruction::Instruction {
    let mut data = Vec::with_capacity(80);
    let auth = from.as_ref();
    // let account =get_token_account!(auth);
    // solana_program::pubkey::Pubkey::find_program_address(&[auth, b"fadfffdasfas"], &id());

    let accounts = vec![
        AccountMeta::new(from.clone(), true),
        AccountMeta::new(get_token_account!(auth, program_id), false),
        AccountMeta::new_readonly(program_id, false),
        AccountMeta::new(get_token_account!(to, program_id), false),
    ];
    data.push(1);
    data.extend_from_slice(&amount.to_be_bytes());
    solana_program::instruction::Instruction::new_with_bytes(program_id, &data, accounts)
}
