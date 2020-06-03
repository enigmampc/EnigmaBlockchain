import { TokenConfiguration } from "@cosmwasm/bcp";

export const binaryName = "cosmwasm-faucet";
export const concurrency: number = Number.parseInt(process.env.FAUCET_CONCURRENCY || "", 10) || 5;
export const port: number = Number.parseInt(process.env.FAUCET_PORT || "", 10) || 8000;
export const mnemonic: string | undefined = process.env.FAUCET_MNEMONIC;

export const addressPrefix = "cosmos";

/** For the local development chain */
export const developmentTokenConfig: TokenConfiguration = {
  bankTokens: [
    {
      fractionalDigits: 6,
      name: "Fee Token",
      ticker: "COSM",
      denom: "ucosm",
    },
    {
      fractionalDigits: 6,
      name: "Staking Token",
      ticker: "STAKE",
      denom: "ustake",
    },
  ],
  erc20Tokens: [
    {
      contractAddress: "cosmos1hqrdl6wstt8qzshwc6mrumpjk9338k0lr4dqxd",
      fractionalDigits: 0,
      ticker: "ISA",
      name: "Isa Token",
    },
  ],
};
