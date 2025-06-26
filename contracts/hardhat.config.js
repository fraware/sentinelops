/**
 * Hardhat configuration for SentinelOps anchor contract.
 *
 * Networks:
 *   • polygon      – main-net  (RPC URL from env POLYGON_RPC)
 *   • polygonTest  – Mumbai    (RPC URL from env POLYGON_RPC_TEST)
 *
 * Deployer key is taken from env POLYGON_PRIVATE_KEY.
 */

require("@nomicfoundation/hardhat-toolbox");     // ethers + waffle + hardhat-verify
require("dotenv").config();

const { POLYGON_RPC, POLYGON_RPC_TEST, POLYGON_PRIVATE_KEY } = process.env;

module.exports = {
  solidity: "0.8.20",
  networks: {
    polygon: {
      url: POLYGON_RPC || "https://polygon-rpc.com",
      accounts: [ POLYGON_PRIVATE_KEY ].filter(Boolean),
      gas: "auto",
      gasPrice: "auto"
    },
    polygonTest: {
      url: POLYGON_RPC_TEST || "https://rpc-mumbai.maticvigil.com",
      accounts: [ POLYGON_PRIVATE_KEY ].filter(Boolean),
      gas: "auto",
      gasPrice: "auto"
    }
  }
};
