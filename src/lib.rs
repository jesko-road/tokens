use std::str::FromStr;

use solana_program::instruction::AccountMeta;
use solana_program::program::invoke_signed;
use solana_program::sysvar::{Sysvar, SysvarId};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::{self, INVALID_INSTRUCTION_DATA},
    pubkey::Pubkey,
};
macro_rules! get_value {
    ($src:ident,$start:expr,$end:expr) => {{
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&(*$src)[$start..$end]);
        u64::from_be_bytes(raw)
    }};
}
macro_rules! get_from_raw_value {
    ($src:ident,$start:expr,$end:expr) => {{
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&$src[$start..$end]);
        u64::from_be_bytes(raw)
    }};
}
macro_rules! write_value {
    ($src:ident,$val:ident,$start:expr,$end:expr) => {
        (*$src)[$start..$end].copy_from_slice(&$val.to_be_bytes())
    };
}

static log_program_id: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("11111111111111111111111111111111");

static SPACE_SIZE: u64 = 80;
// solana_program::declare_id!("AV7gAXgDrYDnbp1AvTQ2q2i39z51eLcFBcVhwGEeCyP3");
// Declare and export the program's entrypoint
solana_program::entrypoint!(token_program);
// Program entrypoint's implementation
pub fn token_program(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    data: &[u8],         // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    let mut accounts = accounts.iter();
    let signer = next_account_info(&mut accounts)?;
    let pay_account = next_account_info(&mut accounts)?;

    // let program_id = next_account_info(&mut accounts)?;
    if *pay_account.owner != program_id.clone() {
        return Err(program_error::ProgramError::IllegalOwner);
    }
    match data[0] {
        0 => {
            let rent_id =
                solana_program::rent::Rent::from_account_info(next_account_info(&mut accounts)?)?;
            let lamports = rent_id.minimum_balance(SPACE_SIZE as usize);

            //init account
            let ix = solana_program::system_instruction::create_account(
                signer.key,
                pay_account.key,
                lamports,
                SPACE_SIZE,
                program_id,
            );

            solana_program::program::invoke_signed(
                &ix,
                &[signer.clone(), pay_account.clone()],
                &[&[&signer.key.as_ref(), &[data[1]]]],
            )?;
        }
        1 => {
            let dest_account = next_account_info(&mut accounts)?;
            if *dest_account.owner != program_id.clone() {
                return Err(program_error::ProgramError::IllegalOwner);
            }
            //transfer account
            let mut raw = [0u8; 8];
            raw.copy_from_slice(&data[1..9]);
            let transfer_amount = u64::from_be_bytes(raw);
            let mut pay_account_raw = pay_account.data.borrow_mut();
            raw.copy_from_slice(&(*pay_account_raw)[0..8]);
            let mut pay_balance = u64::from_be_bytes(raw);
            let mut dest_account_raw = dest_account.data.borrow_mut();
            raw.copy_from_slice(&(*dest_account_raw)[0..8]);
            let mut dest_balance = u64::from_be_bytes(raw);
            if !transfer(&mut pay_balance, &mut dest_balance, transfer_amount) {
                return Err(program_error::ProgramError::InvalidAccountData);
            }
            (pay_account_raw[0..8]).copy_from_slice(&pay_balance.to_be_bytes());
            (dest_account_raw[0..8]).copy_from_slice(&dest_balance.to_be_bytes());
        }
        2 => {
            //airdrop
            let airdrop_amount = {
                let mut raw = [0u8; 8];
                raw.copy_from_slice(&data[1..9]);
                u64::from_be_bytes(raw)
            };
            let mut pay_raw = pay_account.data.borrow_mut();
            let mut balance = get_value!(pay_raw, 0, 8);
            balance += airdrop_amount;
            (*pay_raw)[0..8].copy_from_slice(&balance.to_be_bytes());
        }
        3 => {
            //frozen token

            let amount = get_from_raw_value!(data, 0, 8);
            let mut pay_raw = pay_account.data.borrow_mut();
            let mut balance = get_value!(pay_raw, 0, 8);
            let mut frozen_balance = get_value!(pay_raw, 8, 16);
            if balance <= amount {
                msg!("balance is not enough");
                return Err(program_error::ProgramError::AccountDataTooSmall);
            }
            balance -= amount;
            frozen_balance += amount;
            (*pay_raw)[0..8].copy_from_slice(&balance.to_be_bytes());
            (*pay_raw)[8..16].copy_from_slice(&frozen_balance.to_be_bytes());
        }
        4 => {
            //borrow 16~24,debit 24~32
            let dest_account = next_account_info(&mut accounts)?;
            let mut pay_raw = pay_account.data.borrow_mut();
            let mut dest_raw = dest_account.data.borrow_mut();
            let amount = get_from_raw_value!(data, 1, 9);
            let mut sbrow = get_value!(pay_raw, 16, 24);
            let dbrow = get_value!(dest_raw, 16, 24);
            let debits = get_value!(pay_raw, 24, 32);
            let mut debitd = get_value!(dest_raw, 24, 32);
            let mut scash = get_value!(pay_raw, 0, 8);
            let dcash = get_value!(dest_raw, 0, 8);
            let sav = scash as i64 - debits as i64;
            if sav <= 0 {
                msg!("borrower balance is not enough");
                return Err(program_error::ProgramError::InvalidAccountData);
            }
            let sav = sav as u64;
            let dav = (dcash + dbrow - debitd) as i64;
            if dav <= 0 {
                msg!("dest credit value is not enough");
                return Err(program_error::ProgramError::InvalidAccountData);
            }
            let dav = dav as u64;
            if !(amount < sav && dav > amount) {
                msg!("borrower balance is not enough");
                return Err(program_error::ProgramError::InvalidAccountData);
            }
            scash -= amount;
            sbrow += amount;
            debitd += amount;

            write_value!(pay_raw, scash, 0, 8);
            write_value!(dest_raw, dcash, 0, 8);
            write_value!(pay_raw, sbrow, 16, 24);
            write_value!(dest_raw, dbrow, 16, 24);
            write_value!(pay_raw, debits, 24, 32);
            write_value!(dest_raw, debitd, 24, 32);
            //write log
            write_log(program_id, pay_account, dest_account, amount)?;
        }
        _ => {
            return Err(INVALID_INSTRUCTION_DATA.into());
        }
    }
    return Ok(());
}
fn transfer(pay_balance: &mut u64, dest_balance: &mut u64, amount: u64) -> bool {
    if *pay_balance < amount {
        return false;
    }
    *pay_balance -= amount;
    *dest_balance += amount;
    true
}

fn write_log<'info>(
    program_id: &Pubkey,
    src: &AccountInfo<'info>,
    dst: &AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
    let mut data = [0u8; 10];
    let (a1, a2) = if (src.key.as_ref() >= dst.key.as_ref()) {
        (src.clone(), dst.clone())
    } else {
        data[0] = 1;
        (dst.clone(), src.clone())
    };
    data[1..9].copy_from_slice(&amount.to_be_bytes());
    let (account, bump) = solana_program::pubkey::Pubkey::find_program_address(
        &[a1.key.as_ref(), a2.key.as_ref()],
        program_id,
    );
    data[10] = bump;
    let ix = solana_program::instruction::Instruction::new_with_bytes(
        log_program_id,
        &data,
        vec![
            AccountMeta::new(*a1.key, false),
            AccountMeta::new(*a2.key, false),
            AccountMeta::new(account, false),
            AccountMeta::new_readonly(solana_program::rent::Rent::id(), false),
        ],
    );

    invoke_signed(&ix, &[a1, a2], &[])?;
    Ok(())
}
