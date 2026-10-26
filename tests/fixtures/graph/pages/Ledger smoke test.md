- This is synthetic test data for a read-only diagnostic alpha, not a real account.
- Lunch
  id:: 4a0e7a01-1479-4cb6-b9c8-534635837c2a
  ledger-kind:: expense
  ledger-amount:: 28.00
  ledger-currency:: CNY
  ledger-category:: [[Ledger categories/Food/Meals]]
  ledger-at:: 20261026123000
  ledger-timezone:: +08:00
  ledger-account:: [[Wallet]]
- Own-account transfer
  ledger-kind:: transfer
  ledger-amount:: 100.00
  ledger-currency:: CNY
  ledger-at:: 20261026133000
  ledger-timezone:: +08:00
  ledger-account:: [[Bank]]
  ledger-to-account:: [[Wallet]]
- Refund
  ledger-kind:: refund
  ledger-amount:: 8.00
  ledger-currency:: CNY
  ledger-category:: [[Ledger categories/Food/Meals]]
  ledger-at:: 20261026143000
  ledger-timezone:: +08:00
  ledger-account:: [[Wallet]]
  ledger-refund-of:: ((4a0e7a01-1479-4cb6-b9c8-534635837c2a))
- Invalid amount
  ledger-kind:: expense
  ledger-amount:: -1
  ledger-currency:: CNY
  ledger-category:: [[Ledger categories/Food/Meals]]
  ledger-at:: 20261026153000
  ledger-timezone:: +08:00
  ledger-account:: [[Wallet]]
- A fenced example is not a transaction
  ```text
  ledger-kind:: expense
  ledger-amount:: 28.00
  ledger-currency:: CNY
  ledger-category:: [[Ledger categories/Food/Meals]]
  ledger-at:: 20261026123000
  ledger-timezone:: +08:00
  ledger-account:: [[Wallet]]
  ```
