# Broker permission system
This example mimics a trading permission system. We use Cedar to specify trading permissions of securities including stocks, ETFs, and options.

## Stocks
All users can trade listed stocks.

## ETFs
All users can trade ETFs except that only users with high risk tolerance can trade leveraged and inverse ETFs; only US users can trade crypto-based ETFs.

## Options
Only users with options level greater than 0 can trade options. There are four options levels that permit users to trade options of various risks and complexities.

Note that checking options trading permissions should be a two step process: First check the options trading permissions and then those of the underlying securities. For instance, non-US users cannot trade options of any ETFs tracking crypto currencies.



