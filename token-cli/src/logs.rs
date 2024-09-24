use std::error::Error;

use solana_client::rpc_client;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::Signer,
    sysvar::SysvarId,
};
static BOOK_SIZE: usize = 32 * 40;
static LOG_DATA_SIZE: usize = 9 + 64;
// pub fn init_log_book(log_program_id: &Pubkey, owner: &Pubkey) -> Instruction {
//     let book_account = Keypair::new();
//     solana_sdk::instruction::Instruction::new_with_bytes(
//         log_program_id.clone(),
//         &[1],
//         vec![
//             AccountMeta::new(*owner, true),
//             AccountMeta::new(book_account.pubkey(), false),
//             AccountMeta::new_readonly(solana_program::rent::Rent::id(), false),
//         ],
//     )
// }
pub struct LogData {
    creditor: Pubkey,
    debtor: Pubkey,
    amount: u64,
}
pub fn read_log_book(
    log_program_id: Pubkey,
    c: &rpc_client::RpcClient,
    book: &Pubkey,
) -> Result<Vec<LogData>, Box<dyn Error>> {
    let book_account = c.get_account(book)?;
    if book_account.owner != log_program_id {
        return Err(Box::new(crate::error::Error("account is not book account")));
    } else if book_account.data.len() != BOOK_SIZE {
        return Err(Box::new(crate::error::Error(
            "book account version not match",
        )));
    }
    let mut dd = [0u8; 32];
    let mut result = Vec::with_capacity(32);
    for i in 1..=40 {
        let data = &book_account.data[(i - 1) * 32..i * 32];
        if data
            == &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        {
            break;
        }
        dd.copy_from_slice(data);
        let pubkey = Pubkey::new_from_array(dd);
        result.push(read_log(log_program_id, c, &pubkey)?);
    }
    Ok(result)
}

pub fn read_log(
    log_program_id: Pubkey,
    c: &rpc_client::RpcClient,
    log_key: &Pubkey,
) -> Result<LogData, Box<dyn Error>> {
    let account = c.get_account(log_key)?;
    if account.owner != log_program_id {
        return Err(Box::new(crate::error::Error("account is not book account")));
    } else if account.data.len() != LOG_DATA_SIZE {
        return Err(Box::new(crate::error::Error(
            "log account version not match",
        )));
    }
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&account.data[9..41]);
    let a1 = Pubkey::new_from_array(buff);
    buff.copy_from_slice(&account.data[41..73]);
    let a2 = Pubkey::new_from_array(buff);
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&account.data[1..9]);
    let amount = u64::from_be_bytes(buff);
    match account.data[0] {
        0 => {
            //a1=>a2
            return Ok(LogData {
                creditor: a1,
                debtor: a2,
                amount: amount,
            });
        }
        1 => {
            //a2=>a1
            return Ok(LogData {
                creditor: a2,
                debtor: a1,
                amount: amount,
            });
        }
        _ => {
            return Err(Box::new(crate::error::Error(
                "account data is invalid error",
            )));
        }
    }
}

fn init_log_data_account(
    program_id: &Pubkey,
    c: &rpc_client::RpcClient,
    payer: &Pubkey,
    a1: &Pubkey,
    a2: &Pubkey,
) {
    let (pda, bump) = Pubkey::find_program_address(&[a1.as_ref(), a2.as_ref()], program_id);
    solana_program::instruction::Instruction::new_with_bytes(
        program_id.clone(),
        &[],
        vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(*a1, false),
            AccountMeta::new_readonly(*a2, false),
            AccountMeta::new_readonly(solana_program::rent::Rent::id(), false),
        ],
    );
    // solana_program::system_instruction::create_account();
}
pub fn init_log_book_account(
    program_id: &Pubkey,
    owner: &Pubkey,
) -> (Keypair, solana_program::instruction::Instruction) {
    let kp = Keypair::new();
    let pubkey = kp.pubkey();
    (
        kp,
        solana_program::instruction::Instruction::new_with_bytes(
            *program_id,
            &[],
            vec![
                AccountMeta::new(*owner, true),
                AccountMeta::new(pubkey, false),
                AccountMeta::new_readonly(solana_program::rent::Rent::id(), false),
            ],
        ),
    )
}
