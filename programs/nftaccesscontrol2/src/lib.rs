use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("4qt3yEGTGGMHyDGgxaPeeq1Gz1MpKWqS8t7Sp4nqL97g");

#[program]
pub mod nftaccesscontrol2 {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        license_price: u64,
    ) -> Result<()> {
        let publisher = &mut ctx.accounts.publisher;
        publisher.authority = ctx.accounts.authority.key();
        publisher.license_mint = ctx.accounts.license_mint.key();
        publisher.license_price = license_price;
        publisher.total_subscribers = 0;
        Ok(())
    }

    pub fn purchase_license(
        ctx: Context<PurchaseLicense>,
    ) -> Result<()> {
        let payment_amount = ctx.accounts.publisher.license_price;

        // Transfer payment from subscriber to publisher
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.subscriber.key(),
            &ctx.accounts.authority.key(),
            payment_amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.subscriber.to_account_info(),
                ctx.accounts.authority.to_account_info(),
            ],
        )?;

        // Get the bump seed
        let bump = ctx.bumps.publisher;
        
        // Mint NFT license to subscriber
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.license_mint.to_account_info(),
                    to: ctx.accounts.subscriber_token_account.to_account_info(),
                    authority: ctx.accounts.publisher.to_account_info(),
                },
                &[&[
                    b"publisher",
                    ctx.accounts.authority.key().as_ref(),
                    &[bump],
                ]],
            ),
            1,
        )?;

        // Update subscriber count
        let publisher = &mut ctx.accounts.publisher;
        publisher.total_subscribers += 1;

        Ok(())
    }

    pub fn revoke_license(
        ctx: Context<RevokeLicense>,
    ) -> Result<()> {
        // Burn the NFT license
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.license_mint.to_account_info(),
                    from: ctx.accounts.subscriber_token_account.to_account_info(),
                    authority: ctx.accounts.subscriber.to_account_info(),
                },
            ),
            1,
        )?;

        // Update subscriber count
        let publisher = &mut ctx.accounts.publisher;
        publisher.total_subscribers = publisher.total_subscribers.checked_sub(1)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }

    pub fn verify_access(ctx: Context<VerifyAccess>) -> Result<bool> {
        let token_balance = ctx.accounts.subscriber_token_account.amount;
        Ok(token_balance > 0)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 8,
        seeds = [b"publisher", authority.key().as_ref()],
        bump
    )]
    pub publisher: Account<'info, Publisher>,
    pub license_mint: Account<'info, Mint>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct RevokeLicense<'info> {
    #[account(
        mut,
        seeds = [b"publisher", authority.key().as_ref()],
        bump,
        has_one = authority,
    )]
    pub publisher: Account<'info, Publisher>,
    #[account(mut)]
    pub license_mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is the subscriber's account, which is safe to access in this context.
    pub subscriber: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = license_mint,
        associated_token::authority = subscriber,
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}



#[account]
pub struct Publisher {
    pub authority: Pubkey,
    pub license_mint: Pubkey,
    pub license_price: u64,
    pub total_subscribers: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bump seed not found")]
    BumpSeedNotFound,
    #[msg("Math operation overflow")]
    MathOverflow,
}


#[derive(Accounts)]
pub struct PurchaseLicense<'info> {
    #[account(
        mut,
        seeds = [b"publisher", authority.key().as_ref()],
        bump,
    )]
    pub publisher: Account<'info, Publisher>,
    pub license_mint: Account<'info, Mint>,
    #[account(mut)]
    /// CHECK: This is the publisher authority, validated by the program's logic.
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub subscriber: Signer<'info>,
    #[account(
        init_if_needed,
        payer = subscriber,
        associated_token::mint = license_mint,
        associated_token::authority = subscriber,
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct VerifyAccess<'info> {
    #[account(
        seeds = [b"publisher", authority.key().as_ref()],
        bump,
    )]
    pub publisher: Account<'info, Publisher>,
    /// CHECK: This is the publisher authority; no further checks are necessary as per program requirements.
    pub authority: AccountInfo<'info>,
    /// CHECK: The subscriber account, only required for balance checks; validated externally.
    pub subscriber: AccountInfo<'info>,
    #[account(
        associated_token::mint = license_mint,
        associated_token::authority = subscriber,
    )]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    pub license_mint: Account<'info, Mint>,
}
