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

use borsh::{BorshSerialize, BorshDeserialize};

entrypoint!(process_instruction);

pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let instruction = CounterInstruction::unpack(instruction_data)?;
    match instruction {
        CounterInstruction::InitializeCounter {initial_value} => {
            process_initialize_counter(program_id, accounts, initial_value)?
        }
        CounterInstruction::IncrementCounter => {
            process_increment_counter(program_id, accounts)?
        }
    }
    Ok(())
}

//pub keyword
pub fn process_initialize_counter(program_id: &Pubkey, accounts: &[AccountInfo], initial_value: u64) -> ProgramResult {
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
    };

    let mut account_data = &mut counter_account.data.borrow_mut()[..];
    counter_data.serialize(&mut account_data).unwrap();
    msg!("Counter initialized with initial data {}", initial_value);

    Ok(())
}
pub fn process_increment_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    if *counter_account.owner != *program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut data = counter_account.data.borrow_mut();  //returns a smart pointer
    let mut counter_data : CounterAccount = CounterAccount::try_from_slice(&data)?;
    counter_data.count = counter_data.count.checked_add(1).ok_or(ProgramError::InvalidAccountData)?;
    counter_data.serialize(&mut &mut data[..]).unwrap();     // learned here about smart pointers the deref trait and how i can use the same format as in the increment function inside the initialize function
    Ok(()) 
}   

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    count:u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    InitializeCounter {initial_value: u64},  // variant 0
    IncrementCounter,                        // variant 1
}

impl CounterInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // take the instruction variant from the first byte
        let (&variant, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
        match variant {
            0 => {
                let initial_value = u64::from_le_bytes(rest.try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
                Ok(Self::InitializeCounter {initial_value})     
        }
        1 => {
            Ok(Self::IncrementCounter)
        }
        _ => {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}
}


// testing the contract
#[cfg(test)]
mod test {
    use super::*;
    use solana_program_test::*;
    use solana_program_test::processor;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
    };
#[tokio::test]
async fn test_counter_program() {
    let program_id = Pubkey::new_unique();
    let (mut banks_client, payer, recent_blockhash) = ProgramTest::new("counter_program", program_id, processor!(process_instruction),).start().await;
    let counter_keypair = Keypair::new();
    let initial_val: u64 = 48;
    // Step!: we check for initialization
    println!("Initialising the counter");
    let mut init_instruction_data = vec![0];
    init_instruction_data.extend_from_slice(&initial_val.to_le_bytes());

    let initialize_instruction = Instruction::new_with_bytes(program_id, &init_instruction_data, vec![
        AccountMeta::new(counter_keypair.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(system_program::id(), false),
    ],);

    //sending the transaction
    let mut transaction = Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));
    transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // since the account has now been created we check account data
    let account = banks_client.get_account(counter_keypair.pubkey()).await.expect("Failed to get the account");
    if let Some(account_data) = account {
        let counter : CounterAccount = CounterAccount::try_from_slice(&account_data.data).expect("Failed to deserealize");
        assert_eq!(counter.count , 48);
        println!("Counter initilaized successfully with value {}", counter.count);
    }
}

    //testing the increment instruction
    let mut init_instruction_data2 = vec![1];
    let increment_instruction = Instruction::new_with_bytes(program_id, &init_isntruction_data2, vec![
        AccountMeta::new(counter_keypair.pubkey(), true),
    ]);
    let mut transaction2 = Transaction::new_with_payer(&[increment_instruction], Some(payer.pubkey()));
    transaction.sign(&[&payer, &counter_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    //once again we check whether the incrementation has happened
    let account2 = banks_client.get_account(counter_keypair.pubkey()).await.expect("Failed to get the account");
    if let Some(account_data2) = account2 {
        let counter2 : CounterAccount = CounterAccount::try_from_slice(&account_data2.data).expect("Failed to deserialize");
        asserteq!(counter2.count, 49);
        println!("Counter has been incremented successfully and the current count is {}", counter2.count);
    }
}