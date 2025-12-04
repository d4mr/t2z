package t2z_uniffi

import (
	"strings"
	"testing"
)

func TestVersion(t *testing.T) {
	version := Version()
	if version == "" {
		t.Error("Version() returned empty string")
	}
	if !strings.Contains(version, "t2z") {
		t.Errorf("Version() should contain 't2z', got: %s", version)
	}
	t.Logf("Version: %s", version)
}

func TestIsProvingKeyReady(t *testing.T) {
	// Initially, the proving key should not be ready (unless prebuild was called)
	ready := IsProvingKeyReady()
	t.Logf("IsProvingKeyReady: %v", ready)
	// This is just a sanity check - the value depends on whether prebuild was called
}

func TestUniffiTransparentInputCreation(t *testing.T) {
	// Test creating a transparent input struct
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     nil,
	}

	if input.Pubkey == "" {
		t.Error("Pubkey should not be empty")
	}
	if input.Value != 1000000 {
		t.Errorf("Value should be 1000000, got %d", input.Value)
	}
}

func TestUniffiPaymentCreation(t *testing.T) {
	// Test creating a payment struct
	memo := "48656c6c6f" // "Hello" in hex
	label := "Test Payment"

	payment := UniffiPayment{
		Address: "t1test",
		Amount:  500000,
		Memo:    &memo,
		Label:   &label,
	}

	if payment.Address != "t1test" {
		t.Errorf("Address should be 't1test', got %s", payment.Address)
	}
	if payment.Amount != 500000 {
		t.Errorf("Amount should be 500000, got %d", payment.Amount)
	}
	if payment.Memo == nil || *payment.Memo != memo {
		t.Error("Memo not set correctly")
	}
}

func TestUniffiTransactionRequestCreation(t *testing.T) {
	// Test creating a transaction request
	payment := UniffiPayment{
		Address: "t1test",
		Amount:  500000,
		Memo:    nil,
		Label:   nil,
	}

	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{payment},
	}

	if len(request.Payments) != 1 {
		t.Errorf("Expected 1 payment, got %d", len(request.Payments))
	}
}

func TestUniffiExpectedTxOutCreation(t *testing.T) {
	// Test creating expected tx out for verification
	expected := UniffiExpectedTxOut{
		Address: "t1change",
		Amount:  100000,
	}

	if expected.Address != "t1change" {
		t.Errorf("Address should be 't1change', got %s", expected.Address)
	}
	if expected.Amount != 100000 {
		t.Errorf("Amount should be 100000, got %d", expected.Amount)
	}
}

func TestProposeTransactionInvalidAddress(t *testing.T) {
	// Test that ProposeTransaction returns an error for invalid addresses
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     nil,
	}

	payment := UniffiPayment{
		Address: "invalid_address",
		Amount:  500000,
		Memo:    nil,
		Label:   nil,
	}

	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{payment},
	}

	changeAddr := "also_invalid"
	_, err := ProposeTransaction(
		[]UniffiTransparentInput{input},
		request,
		&changeAddr,
		"testnet",
		3720100,
	)

	if err == nil {
		t.Error("ProposeTransaction should return error for invalid address")
	}

	// Check that the error message is meaningful
	errStr := err.Error()
	if !strings.Contains(errStr, "Invalid") && !strings.Contains(errStr, "invalid") && !strings.Contains(errStr, "NotZcash") {
		t.Errorf("Error should mention invalid address, got: %s", errStr)
	}
	t.Logf("Expected error received: %s", errStr)
}

func TestProposeTransactionInvalidNetwork(t *testing.T) {
	// Test that ProposeTransaction handles network parameter
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     nil,
	}

	payment := UniffiPayment{
		Address: "t1invalid",
		Amount:  500000,
		Memo:    nil,
		Label:   nil,
	}

	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{payment},
	}

	changeAddr := "t1change"
	_, err := ProposeTransaction(
		[]UniffiTransparentInput{input},
		request,
		&changeAddr,
		"invalid_network",
		3720100,
	)

	if err == nil {
		t.Error("ProposeTransaction should return error for invalid network")
	}
	t.Logf("Expected error received: %s", err.Error())
}

func TestProposeTransactionNoChangeAddress(t *testing.T) {
	// Test ProposeTransaction without change address (nil)
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     nil,
	}

	payment := UniffiPayment{
		Address: "invalid_address",
		Amount:  500000,
		Memo:    nil,
		Label:   nil,
	}

	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{payment},
	}

	// Test with nil change address
	_, err := ProposeTransaction(
		[]UniffiTransparentInput{input},
		request,
		nil, // No change address
		"testnet",
		3720100,
	)

	// Should fail due to invalid payment address, not nil change address
	if err == nil {
		t.Error("ProposeTransaction should return error for invalid payment address")
	}
	t.Logf("Expected error received: %s", err.Error())
}

func TestCombinePcztsEmpty(t *testing.T) {
	// Test CombinePczts with empty list
	_, err := CombinePczts([]*UniffiPczt{})
	if err == nil {
		t.Error("CombinePczts should return error for empty list")
	}
	t.Logf("Expected error received: %s", err.Error())
}

func TestMultiplePayments(t *testing.T) {
	// Test creating multiple payments
	payments := []UniffiPayment{
		{Address: "addr1", Amount: 100000, Memo: nil, Label: nil},
		{Address: "addr2", Amount: 200000, Memo: nil, Label: nil},
		{Address: "addr3", Amount: 300000, Memo: nil, Label: nil},
	}

	request := UniffiTransactionRequest{
		Payments: payments,
	}

	if len(request.Payments) != 3 {
		t.Errorf("Expected 3 payments, got %d", len(request.Payments))
	}

	total := uint64(0)
	for _, p := range request.Payments {
		total += p.Amount
	}
	if total != 600000 {
		t.Errorf("Expected total 600000, got %d", total)
	}
}

func TestInputWithSequence(t *testing.T) {
	// Test creating input with custom sequence
	seq := uint32(0xfffffffe)
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     &seq,
	}

	if input.Sequence == nil {
		t.Error("Sequence should not be nil")
	}
	if *input.Sequence != 0xfffffffe {
		t.Errorf("Sequence should be 0xfffffffe, got %x", *input.Sequence)
	}
}

func TestMultipleInputs(t *testing.T) {
	// Test creating multiple inputs
	inputs := []UniffiTransparentInput{
		{
			Pubkey:       "02" + strings.Repeat("aa", 32),
			PrevoutTxid:  strings.Repeat("01", 32),
			PrevoutIndex: 0,
			Value:        500000,
			ScriptPubkey: "76a914" + strings.Repeat("11", 20) + "88ac",
			Sequence:     nil,
		},
		{
			Pubkey:       "02" + strings.Repeat("bb", 32),
			PrevoutTxid:  strings.Repeat("02", 32),
			PrevoutIndex: 1,
			Value:        600000,
			ScriptPubkey: "76a914" + strings.Repeat("22", 20) + "88ac",
			Sequence:     nil,
		},
	}

	if len(inputs) != 2 {
		t.Errorf("Expected 2 inputs, got %d", len(inputs))
	}

	total := uint64(0)
	for _, i := range inputs {
		total += i.Value
	}
	if total != 1100000 {
		t.Errorf("Expected total 1100000, got %d", total)
	}
}

func TestEmptyPayments(t *testing.T) {
	// Test creating request with no payments
	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{},
	}

	if len(request.Payments) != 0 {
		t.Errorf("Expected 0 payments, got %d", len(request.Payments))
	}

	// ProposeTransaction should fail with no payments
	input := UniffiTransparentInput{
		Pubkey:       "02" + strings.Repeat("ab", 32),
		PrevoutTxid:  strings.Repeat("00", 32),
		PrevoutIndex: 0,
		Value:        1000000,
		ScriptPubkey: "76a914" + strings.Repeat("00", 20) + "88ac",
		Sequence:     nil,
	}

	changeAddr := "t1change"
	_, err := ProposeTransaction(
		[]UniffiTransparentInput{input},
		request,
		&changeAddr,
		"testnet",
		3720100,
	)

	if err == nil {
		t.Error("ProposeTransaction should return error for empty payments")
	}
	t.Logf("Expected error received: %s", err.Error())
}

func TestZeroAmount(t *testing.T) {
	// Test payment with zero amount
	payment := UniffiPayment{
		Address: "t1test",
		Amount:  0,
		Memo:    nil,
		Label:   nil,
	}

	if payment.Amount != 0 {
		t.Errorf("Amount should be 0, got %d", payment.Amount)
	}
}

func TestMaxAmount(t *testing.T) {
	// Test payment with max uint64 amount
	payment := UniffiPayment{
		Address: "t1test",
		Amount:  ^uint64(0), // Max uint64
		Memo:    nil,
		Label:   nil,
	}

	if payment.Amount != ^uint64(0) {
		t.Errorf("Amount should be max uint64, got %d", payment.Amount)
	}
}

func TestProposeTransactionEmptyInputs(t *testing.T) {
	// Test ProposeTransaction with empty inputs
	payment := UniffiPayment{
		Address: "t1test",
		Amount:  500000,
		Memo:    nil,
		Label:   nil,
	}

	request := UniffiTransactionRequest{
		Payments: []UniffiPayment{payment},
	}

	changeAddr := "t1change"
	_, err := ProposeTransaction(
		[]UniffiTransparentInput{}, // Empty inputs
		request,
		&changeAddr,
		"testnet",
		3720100,
	)

	if err == nil {
		t.Error("ProposeTransaction should return error for empty inputs")
	}
	t.Logf("Expected error received: %s", err.Error())
}
