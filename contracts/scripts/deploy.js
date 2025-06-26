/**
 * Deployment script:
 *   npx hardhat run scripts/deploy.js --network polygon
 *
 * Writes the deployed address to stdout so the caller can export
 *   export ANCHOR_CONTRACT=0x...
 */

async function main() {
  const Anchor = await ethers.getContractFactory("SentinelAnchor");
  const anchor = await Anchor.deploy();
  await anchor.deployed();
  console.log("SentinelAnchor deployed to", anchor.address);
}

main().catch(e => { console.error(e); process.exit(1); });
