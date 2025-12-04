package main

import (
	"fmt"
	"log"

	t2z "github.com/d4mr/t2z/sdks/go/t2z_uniffi"
)

func main() {
	fmt.Println("t2z-go Example")
	fmt.Println("==============")
	fmt.Println()

	// Check version
	version := t2z.Version()
	fmt.Printf("Library version: %s\n", version)

	// Check proving key status
	ready := t2z.IsProvingKeyReady()
	fmt.Printf("Proving key ready: %v\n", ready)

	// Example: Create a transaction (this will fail without real inputs)
	fmt.Println("\nAttempting to create a transaction...")

	input := t2z.UniffiTransparentInput{
		Pubkey:       "02" + "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
		PrevoutTxid:  "0000000000000000000000000000000000000000000000000000000000000001",
		PrevoutIndex: 0,
		Value:        1000000, // 0.01 ZEC
		ScriptPubkey: "76a914" + "0000000000000000000000000000000000000000" + "88ac",
		Sequence:     nil,
	}

	payment := t2z.UniffiPayment{
		Address: "utest1xxxxxx", // This is not a real address
		Amount:  900000,
		Memo:    nil,
		Label:   nil,
	}

	request := t2z.UniffiTransactionRequest{
		Payments: []t2z.UniffiPayment{payment},
	}

	changeAddr := "utest1change"
	pczt, err := t2z.ProposeTransaction(
		[]t2z.UniffiTransparentInput{input},
		request,
		&changeAddr,
		"testnet",
		3720100,
	)

	if err != nil {
		log.Printf("Expected error (no real inputs): %v\n", err)
	} else {
		fmt.Printf("PCZT created successfully!\n")
		fmt.Printf("PCZT hex: %s...\n", pczt.ToHex()[:64])
	}

	fmt.Println("\nDone!")
}

