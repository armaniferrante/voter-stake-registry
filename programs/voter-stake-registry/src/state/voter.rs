use crate::error::*;
use crate::state::deposit_entry::DepositEntry;
use crate::state::registrar::Registrar;
use anchor_lang::prelude::*;
use spl_governance::state::token_owner_record;

/// User account for minting voting rights.
#[account(zero_copy)]
pub struct Voter {
    pub voter_authority: Pubkey,
    pub registrar: Pubkey,
    pub voter_bump: u8,
    pub voter_weight_record_bump: u8,
    pub deposits: [DepositEntry; 32],

    /// The most recent slot a deposit was made in.
    ///
    /// Would like to use solana_program::clock::Slot here, but Anchor's IDL
    /// does not know the type.
    pub last_deposit_slot: u64,
}

impl Voter {
    pub fn weight(&self, registrar: &Registrar) -> Result<u64> {
        let curr_ts = registrar.clock_unix_timestamp();
        self.deposits
            .iter()
            .filter(|d| d.is_used)
            .try_fold(0, |sum, d| {
                d.voting_power(&registrar.rates[d.rate_idx as usize], curr_ts)
                    .map(|vp| sum + vp)
            })
    }

    pub fn active_deposit_mut(&mut self, index: u8) -> Result<&mut DepositEntry> {
        let index = index as usize;
        require!(index < self.deposits.len(), InvalidDepositEntryIndex);
        let d = &mut self.deposits[index];
        require!(d.is_used, InvalidDepositEntryIndex);
        Ok(d)
    }

    pub fn load_token_owner_record(
        &self,
        account_info: &AccountInfo,
        registrar: &Registrar,
    ) -> Result<token_owner_record::TokenOwnerRecord> {
        let record = token_owner_record::get_token_owner_record_data_for_realm_and_governing_mint(
            &registrar.governance_program_id,
            account_info,
            &registrar.realm,
            &registrar.realm_governing_token_mint,
        )?;
        require!(
            record.governing_token_owner == self.voter_authority,
            InvalidTokenOwnerRecord
        );
        Ok(record)
    }
}