use crate::*;
use ::{
    anchor_spl::token::{ Mint, Token, TokenAccount },
    mpl_token_metadata::{
        accounts::Metadata,
        instructions::{ DelegateStakingV1CpiBuilder, LockV1CpiBuilder },
    },
};

#[derive(Accounts)]
pub struct LockPNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED],
        bump
    )]
    pub global_pool: Account<'info, GlobalPool>,

    pub token_mint: Box<Account<'info, Mint>>, /////////////////////////////////////////

    #[account(
        mut, 
        token::mint = token_mint, 
        token::authority = user,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: instruction will fail if wrong edition is supplied
    pub token_mint_edition: AccountInfo<'info>,
    /// CHECK: instruction will fail if wrong record is supplied
    #[account(mut)]
    pub token_mint_record: AccountInfo<'info>,
    /// CHECK: instruction will fail if wrong metadata is supplied
    #[account(mut)]
    pub mint_metadata: UncheckedAccount<'info>,
    /// CHECK: instruction will fail if wrong rules are supplied
    pub auth_rules: UncheckedAccount<'info>,
    /// CHECK: instruction will fail if wrong sysvar ixns are supplied
    pub sysvar_instructions: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    /// CHECK: intstruction will fail if wrong program is supplied
    pub token_metadata_program: AccountInfo<'info>,                          ///////////////////////////////////////
    /// CHECK: intstruction will fail if wrong program is supplied
    pub auth_rules_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn lock_pnft_handler(ctx: Context<LockPNFT>) -> Result<()> {
    let global_pool = &mut ctx.accounts.global_pool;

    // Verify metadata is legit
    let nft_metadata = Metadata::safe_deserialize(
        &mut ctx.accounts.mint_metadata.to_account_info().data.borrow_mut()
    ).unwrap();

    // Check if this NFT is the wanted collection and verified
    let mut valid: u8 = 0;
    if let Some(collection) = nft_metadata.collection {
        msg!("collection: {}", collection.key.to_string());
        if collection.key.to_string() == "PNFT_COLLECTION_ADDRESS" {
            valid = 1;
        }
    } else {
        return Err(error!(StakingError::MetadataCreatorParseError));
    }
    if let Some(creators) = nft_metadata.creators {
        for creator in creators {
            if creator.address.to_string() == "PNFT_COLLECTION_ADDRESS" {
                valid = 1;
                break;
            }
        }
    } else {
        return Err(error!(StakingError::MetadataCreatorParseError));
    }

    require!(valid == 1, StakingError::InvalidCollection);

    // Lock Pnft to global authority PDA
    let seeds = &[GLOBAL_AUTHORITY_SEED, &[ctx.bumps.global_pool]]; 
    let delegate_seeds = &[&seeds[..]]; 

    DelegateStakingV1CpiBuilder::new(&ctx.accounts.token_metadata_program) //////////////////////////////
        .delegate(&global_pool.to_account_info())
        .metadata(&ctx.accounts.mint_metadata.to_account_info())
        .master_edition(Some(ctx.accounts.token_mint_edition.as_ref())) ////////////////
        .token_record(Some(ctx.accounts.token_mint_record.as_ref())) ///////////////
        .mint(&ctx.accounts.token_mint.to_account_info())
        .token(&ctx.accounts.token_account.to_account_info())
        .authority(&ctx.accounts.user)
        .payer(&ctx.accounts.user)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
        .authorization_rules_program(Some(ctx.accounts.auth_rules_program.as_ref()))
        .authorization_rules(Some(ctx.accounts.auth_rules.as_ref()))
        .amount(1)
        .invoke()?;

    LockV1CpiBuilder::new(&ctx.accounts.token_metadata_program)
        .authority(&global_pool.to_account_info().clone())
        .token_owner(Some(&ctx.accounts.user))
        .token(&ctx.accounts.token_account.to_account_info())
        .mint(&ctx.accounts.token_mint.to_account_info())
        .metadata(&ctx.accounts.mint_metadata)
        .edition(Some(ctx.accounts.token_mint_edition.as_ref()))
        .token_record(Some(ctx.accounts.token_mint_record.as_ref()))
        .payer(&ctx.accounts.user)
        .system_program(&ctx.accounts.system_program)
        .sysvar_instructions(&ctx.accounts.sysvar_instructions)
        .spl_token_program(Some(&ctx.accounts.token_program.to_account_info()))
        .authorization_rules_program(Some(ctx.accounts.auth_rules_program.as_ref()))
        .authorization_rules(Some(ctx.accounts.auth_rules.as_ref()))
        .invoke_signed(delegate_seeds)?;

    global_pool.total_pnft_staked_count += 1;

    Ok(())
}
