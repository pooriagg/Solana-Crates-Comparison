#![cfg_attr(not(test), no_std)]

use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
    ProgramResult,
    program_entrypoint,
    no_allocator,
    nostd_panic_handler,
    program_error::ProgramError,
    log::sol_log
};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

const COUNTER_SIZE: usize = 8;
const INCREMENT: u64 = 1;

fn process_instruction(
    _: &Pubkey,
    accounts_info: &[AccountInfo],
    _: &[u8]
) -> ProgramResult {
    let [ counter_account_info ] = accounts_info else {
        return Err(
            ProgramError::NotEnoughAccountKeys
        );
    };

    if counter_account_info.data_len() != COUNTER_SIZE {
        return Err(
            ProgramError::InvalidAccountData
        );
    };

    let counter_account_data: &mut [u8] = &mut counter_account_info
        .try_borrow_mut_data()
        .unwrap();

    let counter_ptr = counter_account_data.as_mut_ptr() as *mut u64;
    let current_counter: u64 = unsafe { *counter_ptr };

    unsafe {
        counter_ptr.write(
            current_counter
                .checked_add(INCREMENT)
                .unwrap()
        );
    };

    sol_log("Counter incremented.");

    Ok(())
}

#[cfg(test)]
mod test_counter_program {
    use {
        solana_sdk::{
            account::Account as SolanaAccount,
            instruction::{
                AccountMeta,
                Instruction
            },
            pubkey::Pubkey,
            pubkey,
            native_token::sol_to_lamports,
            program_error::ProgramError
        },
        mollusk_svm::{
            result::Check,
            Mollusk
        }
    };

    const COUNTER_PROGRAM_ID: Pubkey = pubkey!("srremy31J5Y25FrAApwVb9kZcfXbusYMMsvTK9aWv5q");

    fn get_mollusk_svm() -> Mollusk {
        Mollusk::new(
            &self::COUNTER_PROGRAM_ID,
            "target/deploy/pinocchio_counter_program"
        )
    }

    fn get_counter_account() -> (Pubkey, SolanaAccount) {
        (
            Pubkey::new_unique(),
            SolanaAccount {
                data: core::primitive::u64::to_le_bytes(2).to_vec(),
                lamports: sol_to_lamports(0.01),
                owner: self::COUNTER_PROGRAM_ID,
                executable: false,
                rent_epoch: 0
            }
        )
    }

    #[test]
    fn test_sucess() {
        let mollusk = self::get_mollusk_svm();

        let counter_account = self::get_counter_account();

        let accounts = [ counter_account.clone() ];
        
        let instruction = Instruction::new_with_bytes(
            self::COUNTER_PROGRAM_ID,
            &[],
            vec![
                AccountMeta::new(counter_account.0, false)
            ]
        );

        let checks = [
            Check::success(),
            Check::account(&counter_account.0)
                .data(&[ 3, 0, 0, 0, 0, 0, 0, 0 ])
                .build()
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &checks
        );
    }

    #[test]
    fn test_failure_no_enough_account_keys() {
        let mollusk = self::get_mollusk_svm();

        let counter_account = self::get_counter_account();
        let random_account = (Pubkey::new_unique(), SolanaAccount::default());

        let accounts = [
            counter_account.clone(),
            random_account.clone()
        ];
        
        let instruction = Instruction::new_with_bytes(
            self::COUNTER_PROGRAM_ID,
            &[],
            vec![
                AccountMeta::new(counter_account.0, false),
                AccountMeta::new_readonly(random_account.0, false)
            ]
        );

        let checks = [
            Check::err(ProgramError::NotEnoughAccountKeys),
            // or Check::instruction_err(solana_sdk::instruction::InstructionError::NotEnoughAccountKeys)
            Check::account(&counter_account.0)
                .data(&[ 2, 0, 0, 0, 0, 0, 0, 0 ])
                .build()
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &checks
        );
    }

    #[test]
    fn test_failure_invalid_account_data() {
        let mollusk = self::get_mollusk_svm();

        let mut counter_account = self::get_counter_account();
        counter_account.1 = SolanaAccount::default();

        let accounts = [ counter_account.clone() ];
        
        let instruction = Instruction::new_with_bytes(
            self::COUNTER_PROGRAM_ID,
            &[],
            vec![
                AccountMeta::new(counter_account.0, false)
            ]
        );

        let checks = [
            Check::err(ProgramError::InvalidAccountData)
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &checks
        );
    }
}