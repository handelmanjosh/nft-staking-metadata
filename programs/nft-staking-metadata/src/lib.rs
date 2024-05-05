use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, metadata::mpl_token_metadata, token::{transfer, Mint, Token, TokenAccount, Transfer}};
declare_id!("2A8PvxwNyoqdfEQKEGAo9PgByToghe4VJcCfcARAHNrd");
const CREATOR1: &str = "46gN6qwoMbZVCJmvVcFbmpy9NNaanPydmqhFy1PCiBfP";
const CREATOR2: &str = "3P7mTbbaFtW13H9txNm9KynxEGLke45iPGCdqgJwGx3c";
const CREATOR3: &str = "BYbFPymSgjC4bxiaDTd9YRxDbcSozUy533Lxy1ozswvZ";
const CREATOR4: &str = "G67VLHRyiCaRgjFgTADw9fjVRuk7MYz45cwer5fBR31U";
const CREATOR5: &str = "4frXBts7pABipQofis4LixoYApPh3VLCFQg2QVYAEQJy";
const CREATOR6: &str = "5PUq4VZSbm5ZwrQMh2bHNLM8ca6UieguKiaLRJqp3d75";
const CREATOR7: &str = "";
#[program]
pub mod nft_staking_metadata {

    use anchor_spl::metadata::mpl_token_metadata::accounts::Metadata;

    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        // msg!("Token state initialized");
        Ok(())
    }
    pub fn create_associated_token_account(_ctx: Context<CreateAssociatedTokenAccount>) -> Result<()> {
        Ok(())
    }
    pub fn fund(ctx: Context<Fund>, amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.program_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info()
                }
            ),
            amount
        )?;
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, collection: u8, size: u64) -> Result<()> {
        if ctx.accounts.stake_account.owner == Pubkey::default() {
            ctx.accounts.stake_account.owner = ctx.accounts.user.key();
        } else {
            if ctx.accounts.stake_account.owner != ctx.accounts.user.key() {
                return Err(CustomError::Unauthorized.into())
            }
        }
        let nft_mint_account_pubkey = ctx.accounts.nft_mint.key();
        let metadata_seed = &[
            "metadata".as_bytes(),
            ctx.accounts.token_metadata_program.key.as_ref(),
            nft_mint_account_pubkey.as_ref()
        ];
        let (metadata_derived_key, _bump) = Pubkey::find_program_address(metadata_seed, ctx.accounts.token_metadata_program.key);
        if  metadata_derived_key != ctx.accounts.nft_metadata_account.key() || ctx.accounts.nft_metadata_account.data_is_empty() {
            return Err(CustomError::IncorrectCollection.into());
        }
        let metadata_full_account =  match Metadata::try_from(&ctx.accounts.nft_metadata_account).ok() {
            None => return Err(CustomError::InvalidAccounts.into()),
            Some(account) => account
        };
        let creators = match metadata_full_account.creators {
            None => return Err(CustomError::InvalidAccounts.into()),
            Some(account) => account,
        };
        let mut valid = false;
        for creator in creators {
            if creator.verified {
                if creator.address == CREATOR1.parse::<Pubkey>().unwrap() 
                || creator.address == CREATOR2.parse::<Pubkey>().unwrap()
                || creator.address == CREATOR3.parse::<Pubkey>().unwrap()
                || creator.address == CREATOR4.parse::<Pubkey>().unwrap()
                || creator.address == CREATOR5.parse::<Pubkey>().unwrap()
                || creator.address == CREATOR6.parse::<Pubkey>().unwrap()
                || creator.address == CREATOR7.parse::<Pubkey>().unwrap() {
                    valid = true;
                }
            }
        }
        if !valid {
            return Err(CustomError::IncorrectCollection.into())
        }
        match ctx.accounts.stake_account.mints.iter().position(|&x| x == ctx.accounts.nft_account.mint) {
            None => -1,
            Some(_index) => return Err(CustomError::AlreadyStaked.into()),
        };
        // let user_info = ctx.accounts.user.to_account_info();
        // let metadata = Metadata::try_from_slice(&ctx.accounts.nft_metadata.data.borrow())?; 
        // if metadata.symbol != "CLB" && metadata.symbol != "UG" && metadata.symbol != "GOTM" && metadata.symbol != "GREATGOATS"
        // && metadata.symbol != "CNDY" {
        //     return Err(CustomError::IncorrectCollection.into())
        // }
        // transfer(
        //     CpiContext::new(
        //         ctx.accounts.token_program.to_account_info(),
        //         Transfer {
        //             from: ctx.accounts.nft_account.to_account_info(),
        //             to: ctx.accounts.stake_token_account.to_account_info(),
        //             authority: user_info,
        //         }
        //     ),
        //     1
        // )?; // remember to add error handling
        let time = Clock::get()?.unix_timestamp;
        if size != ctx.accounts.stake_account.mints.len() as u64 {
            return Err(CustomError::IncorrectSize.into())
        }
        let new_size = StakeInfo::space(ctx.accounts.stake_account.mints.len() + 1);

        let lamports_required = Rent::get()?.minimum_balance(new_size);
        let stake_account_info = ctx.accounts.stake_account.to_account_info();
        if stake_account_info.lamports() < lamports_required {
            let lamports_to_transfer = lamports_required - stake_account_info.lamports();
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.user.key(),
                    stake_account_info.key,
                    lamports_to_transfer
                ),
                &[
                    ctx.accounts.user.to_account_info(),
                    stake_account_info.clone(),
                    ctx.accounts.system_program.to_account_info().clone()
                ]
            )?;
        }
        stake_account_info.realloc(new_size, false)?;
        ctx.accounts.stake_account.add_stake(collection, ctx.accounts.nft_account.mint, time);
        Ok(())
    }
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        // transfer nft from pda
        // close pda
        let stake = &mut ctx.accounts.stake_account;
        if stake.owner != ctx.accounts.user.key() {
            return Err(CustomError::Unauthorized.into())
        }
        // transfer(
        //     CpiContext::new_with_signer(
        //         ctx.accounts.token_program.to_account_info(),
        //         Transfer {
        //             from: ctx.accounts.stake_token_account.to_account_info(),
        //             to: ctx.accounts.nft_account.to_account_info(),
        //             authority: ctx.accounts.program_authority.to_account_info(),
        //         },
        //         &[&[b"auth", &[ctx.bumps.program_authority]]]
        //     ),
        //     1
        // )?;
        let index = match stake.mints.iter().position(|&x| x == ctx.accounts.nft_account.mint) {
            None => return Err(CustomError::NotStaked.into()),
            Some(index) => index,
        };
        let time_diff = Clock::get()?.unix_timestamp - stake.staked_times[index];
        let tokens = (time_diff as u64 * 5 * 10_u64.pow(9)) / 86400; //9 decimals in $UNISIN
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.program_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.program_token_account.to_account_info()
                },
                &[&[b"mint", &[ctx.bumps.program_token_account]]]
            ),
            tokens
        )?;
        stake.remove_stake(index);
        let new_size = StakeInfo::space(ctx.accounts.stake_account.mints.len());
        ctx.accounts.stake_account.to_account_info().realloc(new_size, false)?;
        Ok(())
    }
    pub fn claim<'a, 'b, 'c: 'info, 'info>(ctx: Context<'a, 'b, 'c, 'info, Claim>) -> Result<()> {
        if ctx.accounts.stake_account.owner != ctx.accounts.user.key() {
            return Err(CustomError::Unauthorized.into());
        }
        if ctx.remaining_accounts.len() != ctx.accounts.stake_account.mints.len() {
            return Err(CustomError::InvalidAccounts.into());
        }
        let curr_time = Clock::get()?.unix_timestamp;
        let mut to_remove: Vec<Pubkey> = Vec::new();
        for i in 0..ctx.remaining_accounts.len() {
            let account_info = &ctx.remaining_accounts[i];
            let account = match Account::<TokenAccount>::try_from(account_info).ok() {
                None => return Err(CustomError::InvalidAccounts.into()),
                Some(account) => account,
            };
            if account.owner != ctx.accounts.user.key() || account.mint != ctx.accounts.stake_account.mints[i] {
                return Err(CustomError::InvalidAccounts.into())
            }
            if account.amount != 1 {
                to_remove.push(ctx.accounts.stake_account.mints[i]);
            } else {
                let time_diff = curr_time - ctx.accounts.stake_account.staked_times[i];
                let tokens = (time_diff as u64 * 5 * 10_u64.pow(9)) / 86400; 
                ctx.accounts.stake_account.staked_times[i] = curr_time;
                transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.program_token_account.to_account_info(),
                            to: ctx.accounts.user_token_account.to_account_info(),
                            authority: ctx.accounts.program_token_account.to_account_info()
                        },
                        &[&[b"mint", &[ctx.bumps.program_token_account]]]
                    ),
                    tokens
                )?;
            }
        }
        for mint in &to_remove {
            let index = match ctx.accounts.stake_account.mints.iter().position(|&x| x == *mint) {
                None => return Err(CustomError::InvalidAccounts.into()),
                Some(index) => index,
            };
            ctx.accounts.stake_account.remove_stake(index);
        }
        if to_remove.len() > 0 {
            let new_size = StakeInfo::space(ctx.accounts.stake_account.mints.len());
            ctx.accounts.stake_account.to_account_info().realloc(new_size, false)?;
        }
        Ok(())
    }
}
#[error_code]
pub enum CustomError {
    #[msg("Mint not found")]
    MintNotFound,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Incorrect size")]
    IncorrectSize,
    #[msg("Incorrect collection address")]
    IncorrectCollection,
    #[msg("Invalid Accounts")]
    InvalidAccounts,
    #[msg("Already Staked")]
    AlreadyStaked,
    #[msg("Not Staked")]
    NotStaked
}
#[derive(Accounts)]
pub struct CreateAssociatedTokenAccount<'info> {
    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}
#[derive(Accounts)]
pub struct Fund<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = user,
        token::mint = mint,
        token::authority = program_token_account
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"auth"],
        bump,
        payer = user,
        space = 8
    )]
    /// CHECK: fuck off
    pub program_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeInfo {
    owner: Pubkey,
    collections: Vec<u8>,
    mints: Vec<Pubkey>,
    staked_times: Vec<i64>,
}
impl StakeInfo {
    pub fn add_stake(&mut self, collection: u8, mint: Pubkey, staked_time: i64) {
        self.collections.push(collection);
        self.mints.push(mint);
        self.staked_times.push(staked_time);
    }
    pub fn remove_stake(&mut self, index: usize) {
        self.collections.remove(index);
        self.mints.remove(index);
        self.staked_times.remove(index);
    }   
    pub fn space(num_stakes: usize) -> usize {
        8 + 32 + (4 + num_stakes) + (4 + num_stakes * 32) + (4 + num_stakes * 8)
    }
    pub fn space_external(num_stakes: usize) -> usize {
        let num = num_stakes / 256;
        8 + 32 + (4 + num) + (4 + num * 32) + (4 + num * 8)
    }
}
#[derive(Accounts)]
#[instruction(size: u64)]
pub struct Stake<'info> {
    #[account(
        init_if_needed,
        seeds = [b"stake", user.key().as_ref()],
        bump,
        payer = user,
        space = StakeInfo::space_external(size as usize)

    )]
    pub stake_account: Account<'info, StakeInfo>,
    // #[account(
    //     seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), nft_account.mint.as_ref()],
    //     bump,
    //     seeds::program = mpl_token_metadata::ID,
    // )]
    // /// CHECK:
    // pub nft_metadata: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = nft_account.owner == user.key(),
        constraint = nft_account.amount == 1,
        constraint = nft_account.mint == nft_mint.key(),
    )]
    pub nft_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    pub nft_metadata_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeInfo>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub nft_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"auth"],
        bump,
    )]
    /// CHECK:
    pub program_authority: AccountInfo<'info>,
}


#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"stake", user.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeInfo>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

