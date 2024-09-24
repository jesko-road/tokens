use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::{self, ProgramResult},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
mod error;
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
// #[cfg(testnet)]
static tokens_program_id: Pubkey =
    solana_program::pubkey!("9zuZUTkJBdrp6zZ6uEDjVjUHZT18rKcjchdqQYqyiy4C");
// static SPACE_SIZE: u64 = 9;
static BOOK_SIZE: u64 = 32 * 40;
static LOG_DATA_SIZE: usize = 9 + 64;
// Declare and export the program's entrypoint
solana_program::entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    data: &[u8],         // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    let mut accounts = accounts.iter();
    let signer = next_account_info(&mut accounts)?;
    // let src = next_account_info(&mut accounts)?;
    match data[0] {
        0 => {
            //init log data account
            let pda = next_account_info(&mut accounts)?;
            let a1 = next_account_info(&mut accounts)?;
            let a2 = next_account_info(&mut accounts)?;
            let rent_account = next_account_info(&mut accounts)?;
            create_account(program_id, rent_account, signer, a1, a2, pda, data[1])?;
        }
        1 => {
            let bump = data[9];
            //record borrow log
            // let dst = next_account_info(&mut accounts)?;
            let pda = next_account_info(&mut accounts)?;
            let a1 = next_account_info(&mut accounts)?;
            let a2 = next_account_info(&mut accounts)?;
            let rent_account = next_account_info(&mut accounts)?;
            // let public_account = next_account_info(&mut accounts)?;
            let src_data = a1.key.as_ref();
            let dst_data = a2.key.as_ref();
            if src_data < dst_data {
                msg!("src data bytes must greater than dst data bytes");
                return Err(ProgramError::InvalidArgument);
            }
            // let line = next_account_info(&mut accounts)?;
            let line_len = pda.data_len();
            if line_len == 0 {
                let rent_account = next_account_info(&mut accounts)?;
                //create account
                create_account(
                    program_id,
                    rent_account,
                    signer,
                    a1,
                    a2,
                    pda,
                    bump,
                )?;
                // return Err(ProgramError::UninitializedAccount);
            } else if line_len != LOG_DATA_SIZE {
                msg!("log data space not is log data size 73");
                return Err(ProgramError::InvalidAccountData);
            }
            let mode = data[0];
            let amount = get_from_raw_value!(data, 1, 9);
            let mut line_raw = a2.data.borrow_mut();
            let mut value = get_from_raw_value!(line_raw, 1, 9);
            if line_raw[0] == mode {
                //same forward
                value += amount;
            } else {
                //different forward
                value = if value < amount {
                    line_raw[0] = 1 - line_raw[0];
                    amount - value
                } else {
                    value - amount
                }
            }
            write_value!(line_raw, value, 1, 9);

            line_raw[9..41].copy_from_slice(src_data);
            line_raw[41..73].copy_from_slice(dst_data);
        }
        2 => {
            //init record book
            //32 bytes/per other pubkey
            let book = next_account_info(&mut accounts)?;
            let rent_account = next_account_info(&mut accounts)?;
            let book_len = book.data_len();
            if book_len == BOOK_SIZE as usize && book.owner == program_id {
                return Ok(());
            } else if book_len != 0 || book.owner != program_id {
                msg!("book is init or book owner is log program");
                return Err(ProgramError::InvalidArgument);
            }
            let rt = solana_program::rent::Rent::from_account_info(rent_account)?;
            let lamports = rt.minimum_balance(BOOK_SIZE as usize);
            let ix = solana_program::system_instruction::create_account(
                signer.key, book.key, lamports, BOOK_SIZE, program_id,
            );
            solana_program::program::invoke(&ix, &[signer.clone(), book.clone()])?;
        }
        _ => return Err(ProgramError::InvalidArgument),
    }

    Ok(())
}

fn create_account<'info>(
    program_id: &Pubkey,
    rent_account: &AccountInfo<'info>,
    signer: &AccountInfo<'info>,
    a1: &AccountInfo<'info>,
    a2: &AccountInfo<'info>,
    account: &AccountInfo<'info>,
    bump: u8,
) -> ProgramResult {
    if a1.key.as_ref() < a2.key.as_ref() {
        msg!("a1 must greater than a2");
        return Err(ProgramError::InvalidArgument);
    }
    let rt = solana_program::rent::Rent::from_account_info(rent_account)?;
    let ix = solana_program::system_instruction::create_account(
        &signer.key,
        &account.key,
        rt.minimum_balance(LOG_DATA_SIZE),
        LOG_DATA_SIZE as u64,
        program_id,
    );
    solana_program::program::invoke_signed(
        &ix,
        &[signer.clone(), account.clone()],
        &[&[a1.key.as_ref(), a2.key.as_ref(), &[bump]]],
    )?;
    Ok(())
}
