# Serum-TWAP

Serum-TWAP is a rust application to calculate the Time Weighted Average Price (TWAP) using Bonfida's API to fetch historical serum trades. 
## Usage
Serum-TWAP takes in a symbol (BTC/USD) and an optional interval in minutes (default is 24h or 1440min). 
| Arguement | Required  | Description |
| --- | --- | --- |
| symbol | Y  | The Pyth symbol to calculate the TWAP for. See https://pyth.network/markets |
| interval | N | The interval to calculate the TWAP over in minutes. Default value is 60. |
| debug | N | Flag to turn on verbose logging |

For more help run
```bash
pyth-twap --help
```
### Basic
This example will calulcate the TWAP for BTC/USD over a 1d interval.
```bash
serum-twap BTC/USD
```
### Advanced
This example will calculate the TWAP for DOGE/USD over a 15m interval.
```bash
serum-twap DOGE/USD -i 15
```
