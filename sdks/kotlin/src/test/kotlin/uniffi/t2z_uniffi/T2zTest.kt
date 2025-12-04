package uniffi.t2z_uniffi

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import kotlin.test.assertEquals
import kotlin.test.assertContains
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class T2zTest {

    @Test
    fun testVersion() {
        val version = version()
        assertNotNull(version)
        assertContains(version, "t2z")
        println("Version: $version")
    }

    @Test
    fun testIsProvingKeyReady() {
        // Just check it doesn't throw
        val ready = isProvingKeyReady()
        println("IsProvingKeyReady: $ready")
    }

    @Test
    fun testUniffiTransparentInputCreation() {
        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = null
        )

        assertEquals("02" + "ab".repeat(32), input.pubkey)
        assertEquals(1000000uL, input.value)
    }

    @Test
    fun testUniffiPaymentCreation() {
        val memo = "48656c6c6f" // "Hello" in hex
        val label = "Test Payment"

        val payment = UniffiPayment(
            address = "t1test",
            amount = 500000uL,
            memo = memo,
            label = label
        )

        assertEquals("t1test", payment.address)
        assertEquals(500000uL, payment.amount)
        assertEquals(memo, payment.memo)
    }

    @Test
    fun testUniffiTransactionRequestCreation() {
        val payment = UniffiPayment(
            address = "t1test",
            amount = 500000uL,
            memo = null,
            label = null
        )

        val request = UniffiTransactionRequest(
            payments = listOf(payment)
        )

        assertEquals(1, request.payments.size)
    }

    @Test
    fun testUniffiExpectedTxOutCreation() {
        val expected = UniffiExpectedTxOut(
            address = "t1change",
            amount = 100000uL
        )

        assertEquals("t1change", expected.address)
        assertEquals(100000uL, expected.amount)
    }

    @Test
    fun testProposeTransactionInvalidAddress() {
        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = null
        )

        val payment = UniffiPayment(
            address = "invalid_address",
            amount = 500000uL,
            memo = null,
            label = null
        )

        val request = UniffiTransactionRequest(
            payments = listOf(payment)
        )

        val exception = assertThrows<UniffiException> {
            proposeTransaction(
                inputsToSpend = listOf(input),
                transactionRequest = request,
                changeAddress = "also_invalid",
                network = "testnet",
                expiryHeight = 3720100u
            )
        }

        val message = exception.message ?: ""
        assertTrue(
            message.contains("Invalid") || message.contains("NotZcash"),
            "Error should mention invalid address, got: $message"
        )
        println("Expected error received: $message")
    }

    @Test
    fun testProposeTransactionInvalidNetwork() {
        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = null
        )

        val payment = UniffiPayment(
            address = "t1invalid",
            amount = 500000uL,
            memo = null,
            label = null
        )

        val request = UniffiTransactionRequest(
            payments = listOf(payment)
        )

        val exception = assertThrows<UniffiException> {
            proposeTransaction(
                inputsToSpend = listOf(input),
                transactionRequest = request,
                changeAddress = "t1change",
                network = "invalid_network",
                expiryHeight = 3720100u
            )
        }

        println("Expected error received: ${exception.message}")
    }

    @Test
    fun testProposeTransactionNoChangeAddress() {
        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = null
        )

        val payment = UniffiPayment(
            address = "invalid_address",
            amount = 500000uL,
            memo = null,
            label = null
        )

        val request = UniffiTransactionRequest(
            payments = listOf(payment)
        )

        val exception = assertThrows<UniffiException> {
            proposeTransaction(
                inputsToSpend = listOf(input),
                transactionRequest = request,
                changeAddress = null,
                network = "testnet",
                expiryHeight = 3720100u
            )
        }

        println("Expected error received: ${exception.message}")
    }

    @Test
    fun testCombinePcztsEmpty() {
        val exception = assertThrows<UniffiException> {
            combinePczts(emptyList())
        }

        println("Expected error received: ${exception.message}")
    }

    @Test
    fun testMultiplePayments() {
        val payments = listOf(
            UniffiPayment(address = "addr1", amount = 100000uL, memo = null, label = null),
            UniffiPayment(address = "addr2", amount = 200000uL, memo = null, label = null),
            UniffiPayment(address = "addr3", amount = 300000uL, memo = null, label = null)
        )

        val request = UniffiTransactionRequest(payments = payments)

        assertEquals(3, request.payments.size)

        val total = request.payments.sumOf { it.amount }
        assertEquals(600000uL, total)
    }

    @Test
    fun testInputWithSequence() {
        val seq = 0xfffffffeu
        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = seq
        )

        assertEquals(0xfffffffeu, input.sequence)
    }

    @Test
    fun testMultipleInputs() {
        val inputs = listOf(
            UniffiTransparentInput(
                pubkey = "02" + "aa".repeat(32),
                prevoutTxid = "01".repeat(32),
                prevoutIndex = 0u,
                value = 500000uL,
                scriptPubkey = "76a914" + "11".repeat(20) + "88ac",
                sequence = null
            ),
            UniffiTransparentInput(
                pubkey = "02" + "bb".repeat(32),
                prevoutTxid = "02".repeat(32),
                prevoutIndex = 1u,
                value = 600000uL,
                scriptPubkey = "76a914" + "22".repeat(20) + "88ac",
                sequence = null
            )
        )

        assertEquals(2, inputs.size)

        val total = inputs.sumOf { it.value }
        assertEquals(1100000uL, total)
    }

    @Test
    fun testEmptyPayments() {
        val request = UniffiTransactionRequest(payments = emptyList())

        assertEquals(0, request.payments.size)

        val input = UniffiTransparentInput(
            pubkey = "02" + "ab".repeat(32),
            prevoutTxid = "00".repeat(32),
            prevoutIndex = 0u,
            value = 1000000uL,
            scriptPubkey = "76a914" + "00".repeat(20) + "88ac",
            sequence = null
        )

        val exception = assertThrows<UniffiException> {
            proposeTransaction(
                inputsToSpend = listOf(input),
                transactionRequest = request,
                changeAddress = "t1change",
                network = "testnet",
                expiryHeight = 3720100u
            )
        }

        println("Expected error received: ${exception.message}")
    }

    @Test
    fun testZeroAmount() {
        val payment = UniffiPayment(
            address = "t1test",
            amount = 0uL,
            memo = null,
            label = null
        )

        assertEquals(0uL, payment.amount)
    }

    @Test
    fun testMaxAmount() {
        val payment = UniffiPayment(
            address = "t1test",
            amount = ULong.MAX_VALUE,
            memo = null,
            label = null
        )

        assertEquals(ULong.MAX_VALUE, payment.amount)
    }

    @Test
    fun testProposeTransactionEmptyInputs() {
        val payment = UniffiPayment(
            address = "t1test",
            amount = 500000uL,
            memo = null,
            label = null
        )

        val request = UniffiTransactionRequest(
            payments = listOf(payment)
        )

        val exception = assertThrows<UniffiException> {
            proposeTransaction(
                inputsToSpend = emptyList(),
                transactionRequest = request,
                changeAddress = "t1change",
                network = "testnet",
                expiryHeight = 3720100u
            )
        }

        println("Expected error received: ${exception.message}")
    }
}

