use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

use borsh::{BorshSerialize, BorshDesrialize};

entrypoint!(process_instruction);

pub fn process_instruction(program_id: &PubKey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let instruction = CounterInstruction::unpack(instruction_data)?;
    match instruction {
        CounterInstruction::InitializeCounter {initial_value} => {
            process_initialize_counter(program_id, accounts, initial_value)?
        }
        CounterInstruction::IncrementCounter => {
            process_increment_counter(program_id, accounts, initial_value)?
        }
    }
    Ok(())
}

//pub keyword
pub fn process_initialize_counter(program_id: &PubKey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let account_space = 8;
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(account_space);

    invoke(
        &system_instruction::create_account(payer_account.key, counter_account.key, required_lamports, account_space as u64, program_id),
        &[payer_account.clone(), counter_account.clone(), system_program.clone()],
    )?;

    let counter_data = CounterAccount {
        count : initial_value,
    }

    let mut account_data = &mut counter_account.data.borrow_mut()[..];
    counter_data.serialize(&mut account_data);
    msg!("Counter initialized with initial data {}", initial_value);

    Ok(())
}
pub fn process_increment_counter(program_id: &PubKey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut data = counter_account.data.borrow_mut();  //returns a smart pointer
    let mut counter_data : CounterAccount = CounterAccount::try_from_slice(&data);
    counter_data.count = counter_data.count.checked_add(1).ok_or(ProgramError::InvalidAccountData)?;
    counter_data.serialize(&mut &mut data[..]);     // learned here about smart pointers the deref trait and how i can use the same format as in the increment function inside the initialize function
    Ok(()) 
}   

#[derive(BorshSerialize, BorshDesrialize, Debug)]
pub struct CounterAccount {
    count:u64,
}

#[derive(BorshSerialize, BorshDesrialize, Debug)]
pub enum CounterInstruction {
    InitializeCounter {initial_value: u64},  // variant 0
    IncrementCounter,                        // variant 1
}

impl CounterInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // take the instruction variant from the first byte
        let (&variant, rest) = input.split_first().ok_er(ProgramError::InvalidInstructionData)?;
        match variant {
            0 => {
                let initial_value = u64::from_le_bytes(rest.try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
            }
            Ok(Self::InitializeCounter {initial_value});
        }
        1 => {
            Ok(Self::IncrementCounter)
        }
        _ => {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}