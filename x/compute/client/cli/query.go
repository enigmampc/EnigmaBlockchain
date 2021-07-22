package cli

import (
	"bytes"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"strconv"

	"github.com/spf13/viper"

	cosmwasmTypes "github.com/enigmampc/SecretNetwork/go-cosmwasm/types"
	sdkErrors "github.com/enigmampc/cosmos-sdk/types/errors"
	flag "github.com/spf13/pflag"
	"github.com/tendermint/go-amino"

	"github.com/spf13/cobra"

	"github.com/enigmampc/cosmos-sdk/client"
	"github.com/enigmampc/cosmos-sdk/client/context"
	"github.com/enigmampc/cosmos-sdk/client/flags"
	"github.com/enigmampc/cosmos-sdk/codec"
	sdk "github.com/enigmampc/cosmos-sdk/types"
	"github.com/enigmampc/cosmos-sdk/x/auth/client/utils"

	wasmUtils "github.com/enigmampc/SecretNetwork/x/compute/client/utils"

	"github.com/enigmampc/SecretNetwork/x/compute/internal/keeper"
	"github.com/enigmampc/SecretNetwork/x/compute/internal/types"
)

func GetQueryCmd(cdc *codec.Codec) *cobra.Command {
	queryCmd := &cobra.Command{
		Use:                        types.ModuleName,
		Short:                      "Querying commands for the compute module",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}
	queryCmd.AddCommand(flags.GetCommands(
		GetCmdListCode(cdc),
		GetCmdListContractByCode(cdc),
		GetCmdQueryCode(cdc),
		GetCmdGetContractInfo(cdc),
		GetCmdQuery(cdc),
		GetQueryDecryptTxCmd(cdc),
		GetCmdQueryLabel(cdc),
		GetCmdCodeHashByContract(cdc),
		CmdDecryptText(cdc),
		// GetCmdGetContractHistory(cdc),
	)...)
	return queryCmd
}

// GetCmdListCode lists all wasm code uploaded
func GetCmdListCode(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "list-code",
		Short: "List all wasm bytecode on the chain",
		Long:  "List all wasm bytecode on the chain",
		Args:  cobra.ExactArgs(0),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			route := fmt.Sprintf("custom/%s/%s", types.QuerierRoute, keeper.QueryListCode)
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return err
			}
			fmt.Println(string(res))
			return nil
		},
	}
}

// GetCmdListCode lists all wasm code uploaded
func GetCmdQueryLabel(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "label [label]",
		Short: "Check if a label is in use",
		Long:  "Check if a label is in use",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryContractAddress, args[0])
			res, _, err := cliCtx.Query(route)
			if err != nil {
				if err == sdkErrors.ErrUnknownAddress {
					fmt.Printf("Label is available and not in use\n")
					return nil
				}

				return fmt.Errorf("error querying: %s", err)
			}

			addr := sdk.AccAddress{}

			err = addr.Unmarshal(res)
			if err != nil {
				return fmt.Errorf("error unwrapping address: %s", err)
			}

			fmt.Printf("Label is in use by contract address: %s\n", addr)
			return nil
		},
	}
}

// GetCmdListCode lists all wasm code uploaded
func GetCmdCodeHashByContract(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "contract-hash [address]",
		Short: "Return the code hash of a contract",
		Long:  "Return the code hash of a contract",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryContractHash, args[0])
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return fmt.Errorf("error querying contract hash: %s", err)
			}

			addr := hex.EncodeToString(res)
			fmt.Printf("0x%s", addr)
			return nil
		},
	}
}

// GetCmdListContractByCode lists all wasm code uploaded for given code id
func GetCmdListContractByCode(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "list-contract-by-code [code_id]",
		Short: "List wasm all bytecode on the chain for given code id",
		Long:  "List wasm all bytecode on the chain for given code id",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			codeID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			route := fmt.Sprintf("custom/%s/%s/%d", types.QuerierRoute, keeper.QueryListContractByCode, codeID)
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return err
			}
			fmt.Println(string(res))
			return nil
		},
	}
}

// GetCmdQueryCode returns the bytecode for a given contract
func GetCmdQueryCode(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "code [code_id] [output filename]",
		Short: "Downloads wasm bytecode for given code id",
		Long:  "Downloads wasm bytecode for given code id",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			codeID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			route := fmt.Sprintf("custom/%s/%s/%d", types.QuerierRoute, keeper.QueryGetCode, codeID)
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return err
			}
			if len(res) == 0 {
				return fmt.Errorf("contract not found")
			}
			var code keeper.GetCodeResponse
			err = json.Unmarshal(res, &code)
			if err != nil {
				return err
			}

			if len(code.Data) == 0 {
				return fmt.Errorf("contract not found")
			}

			fmt.Printf("Downloading wasm code to %s\n", args[1])
			return ioutil.WriteFile(args[1], code.Data, 0644)
		},
	}
}

// GetCmdGetContractInfo gets details about a given contract
func GetCmdGetContractInfo(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "contract [bech32_address]",
		Short: "Prints out metadata of a contract given its address",
		Long:  "Prints out metadata of a contract given its address",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			addr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryGetContract, addr.String())
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return err
			}
			fmt.Println(string(res))
			return nil
		},
	}
}

func CmdDecryptText(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "decrypt [encrypted_data]",
		Short: "Attempt to decrypt an encrypted blob",
		Long: "Attempt to decrypt a base-64 encoded encrypted message. This is intended to be used if manual decrypt" +
			"is required for data that is unavailable to be decrypted using the 'query compute tx' command",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			encodedInput := args[0]

			dataCipherBz, err := base64.StdEncoding.DecodeString(encodedInput)
			if err != nil {
				return fmt.Errorf("error while trying to decode the encrypted output data from base64: %w", err)
			}

			nonce := dataCipherBz[0:32]
			originalTxSenderPubkey := dataCipherBz[32:64]

			wasmCtx := wasmUtils.WASMContext{CLIContext: cliCtx}
			_, myPubkey, err := wasmCtx.GetTxSenderKeyPair()

			if !bytes.Equal(originalTxSenderPubkey, myPubkey) {
				return fmt.Errorf("cannot decrypt, not original tx sender")
			}

			dataPlaintextB64Bz, err := wasmCtx.Decrypt(dataCipherBz[64:], nonce)
			if err != nil {
				return fmt.Errorf("error while trying to decrypt the output data: %w", err)
			}

			fmt.Printf("Decrypted data: %s", dataPlaintextB64Bz)
			return nil
		},
	}
}

// QueryDecryptTxCmd the default command for a tx query + IO decryption if I'm the tx sender.
// Coppied from https://github.com/enigmampc/cosmos-sdk/blob/v0.38.4/x/auth/client/cli/query.go#L157-L184 and added IO decryption (Could not wrap it because it prints directly to stdout)
func GetQueryDecryptTxCmd(cdc *amino.Codec) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tx [hash]",
		Short: "Query for a transaction by hash in a committed block, decrypt input and outputs if I'm the tx sender",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			result, err := utils.QueryTx(cliCtx, args[0])
			if err != nil {
				return err
			}

			if result.Empty() {
				return fmt.Errorf("no transaction found with hash %s", args[0])
			}

			var answer struct {
				Type               string                 `json:"type"`
				RawInput           string                 `json:"raw_input"`
				Input              json.RawMessage        `json:"input"`
				OutputData         string                 `json:"output_data"`
				OutputDataAsString string                 `json:"output_data_as_string"`
				OutputLogs         []sdk.StringEvent      `json:"output_log"`
				OutputError        cosmwasmTypes.StdError `json:"output_error"`
				PlaintextError     string                 `json:"plaintext_error"`
			}
			var encryptedInput []byte
			var dataOutputHex string

			txInputs := result.Tx.GetMsgs()
			if len(txInputs) != 1 {
				return fmt.Errorf("can only decrypt txs with 1 input. Got %d", len(txInputs))
			}
			txInput := txInputs[0]

			if txInput.Type() == "execute" {
				execTx, ok := txInput.(types.MsgExecuteContract)
				if !ok {
					return fmt.Errorf("error parsing tx as type 'execute': %v", txInput)
				}

				encryptedInput = execTx.Msg
				dataOutputHex = result.Data
			} else if txInput.Type() == "instantiate" {
				initTx, ok := txInput.(types.MsgInstantiateContract)
				if !ok {
					return fmt.Errorf("error parsing tx as type 'instantiate': %v", txInput)
				}

				encryptedInput = initTx.InitMsg
			} else {
				return fmt.Errorf("tx %s is not of type 'execute' or 'instantiate'. Got type '%s'", args[0], txInput.Type())
			}
			answer.Type = txInput.Type()

			// decrypt input
			if len(encryptedInput) < 64 {
				return fmt.Errorf("input must be > 64 bytes. Got %d", len(encryptedInput))
			}

			nonce := encryptedInput[0:32]
			originalTxSenderPubkey := encryptedInput[32:64]

			wasmCtx := wasmUtils.WASMContext{CLIContext: cliCtx}
			_, myPubkey, err := wasmCtx.GetTxSenderKeyPair()
			if err != nil {
				return fmt.Errorf("error in GetTxSenderKeyPair: %w", err)
			}

			if !bytes.Equal(originalTxSenderPubkey, myPubkey) {
				return fmt.Errorf("cannot decrypt, not original tx sender")
			}

			ciphertextInput := encryptedInput[64:]
			var plaintextInput []byte
			if len(ciphertextInput) > 0 {
				plaintextInput, err = wasmCtx.Decrypt(ciphertextInput, nonce)
				if err != nil {
					return fmt.Errorf("error while trying to decrypt the tx input: %w", err)
				}
			}

			answer.RawInput = string(plaintextInput)
			_ = json.Unmarshal(plaintextInput[64:], answer.Input)

			// decrypt data
			if answer.Type == "execute" {
				dataOutputCipherBz, err := hex.DecodeString(dataOutputHex)
				if err != nil {
					return fmt.Errorf("error while trying to decode the encrypted output data from hex string: %w", err)
				}

				dataPlaintextB64Bz, err := wasmCtx.Decrypt(dataOutputCipherBz, nonce)
				if err != nil {
					return fmt.Errorf("error while trying to decrypt the output data: %w", err)
				}
				dataPlaintextB64 := string(dataPlaintextB64Bz)
				answer.OutputData = dataPlaintextB64

				dataPlaintext, err := base64.StdEncoding.DecodeString(dataPlaintextB64)
				if err != nil {
					return fmt.Errorf("error while trying to decode the decrypted output data from base64: %w", err)
				}

				answer.OutputDataAsString = string(dataPlaintext)
			}

			// decrypt logs
			answer.OutputLogs = []sdk.StringEvent{}
			for _, l := range result.Logs {
				for _, e := range l.Events {
					if e.Type == "wasm" {
						for i, a := range e.Attributes {
							if a.Key != "contract_address" {
								// key
								if a.Key != "" {
									// Try to decrypt the log key. If it doesn't look encrypted, leave it as-is
									keyCiphertext, err := base64.StdEncoding.DecodeString(a.Key)
									if err == nil {
										keyPlaintext, err := wasmCtx.Decrypt(keyCiphertext, nonce)
										if err == nil {
											a.Key = string(keyPlaintext)
										}
									}
								}

								// value
								if a.Value != "" {
									// Try to decrypt the log value. If it doesn't look encrypted, leave it as-is
									valueCiphertext, err := base64.StdEncoding.DecodeString(a.Value)
									if err == nil {
										valuePlaintext, err := wasmCtx.Decrypt(valueCiphertext, nonce)
										if err == nil {
											a.Value = string(valuePlaintext)
										}
									}
								}

								e.Attributes[i] = a
							}
						}
						answer.OutputLogs = append(answer.OutputLogs, e)
					}
				}
			}

			if types.IsEncryptedErrorCode(result.Code) && types.ContainsEncryptedString(result.RawLog) {
				stdErr, err := wasmCtx.DecryptError(result.RawLog, answer.Type, nonce)
				if err != nil {
					return err
				}

				answer.OutputError = stdErr
			} else if types.ContainsEnclaveError(result.RawLog) {
				answer.PlaintextError = result.RawLog
			}

			return cliCtx.PrintOutput(answer)
		},
	}

	return cmd
}

func GetCmdQuery(cdc *codec.Codec) *cobra.Command {
	decoder := newArgDecoder(asciiDecodeString)

	cmd := &cobra.Command{
		Use:   "query [bech32_address] [query]", // TODO add --from wallet
		Short: "Calls contract with given address with query data and prints the returned result",
		Long:  "Calls contract with given address with query data and prints the returned result",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			var msg string
			var contractAddr string

			if len(args) == 1 {

				label := viper.GetString(flagLabel)
				if label == "" {
					return fmt.Errorf("label or bech32 contract address is required")
				}

				route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryContractAddress, label)
				res, _, err := cliCtx.Query(route)
				if err != nil {
					return err
				}

				contractAddr = string(res)
				msg = args[0]
			} else {
				// get the id of the code to instantiate
				contractAddr = args[0]
				msg = args[1]
			}

			queryData, err := decoder.DecodeString(msg)
			if err != nil {
				return fmt.Errorf("decode query: %s", err)
			}

			if !json.Valid(queryData) {
				return errors.New("query data must be json")
			}

			return QueryWithData(contractAddr, cliCtx, queryData)
		},
	}
	decoder.RegisterFlags(cmd.PersistentFlags(), "query argument")
	cmd.Flags().String(flagLabel, "", "A human-readable name for this contract in lists")
	return cmd
}

func QueryWithData(contractAddress string, cliCtx context.CLIContext, queryData []byte) error {
	addr, err := sdk.AccAddressFromBech32(contractAddress)
	if err != nil {
		return err
	}

	route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryGetContractState, addr.String())

	wasmCtx := wasmUtils.WASMContext{CLIContext: cliCtx}

	codeHash, err := GetCodeHashByContractAddr(cliCtx, addr)
	if err != nil {
		return fmt.Errorf("contract not found: %s", addr)
	}

	msg := types.SecretMsg{
		CodeHash: codeHash,
		Msg:      queryData,
	}

	queryData, err = wasmCtx.Encrypt(msg.Serialize())
	if err != nil {
		return err
	}
	nonce := queryData[:32]

	res, _, err := cliCtx.QueryWithData(route, queryData)

	if err != nil {
		if types.ErrContainsQueryError(err) {
			errorPlainBz, err := wasmCtx.DecryptError(err.Error(), "query", nonce)
			if err != nil {
				return err
			}
			return fmt.Errorf("query result: %v", errorPlainBz.Error())
		}
		// Itzik: Commenting this as it might have been a placeholder for encrypting
		//else if strings.Contains(err.Error(), "EnclaveErr") {
		//	return err
		//}
		return err
	}

	var resDecrypted []byte
	if len(res) > 0 {
		resDecrypted, err = wasmCtx.Decrypt(res, nonce)
		if err != nil {
			return err
		}
	}

	decodedResp, err := base64.StdEncoding.DecodeString(string(resDecrypted))
	if err != nil {
		return err
	}

	fmt.Println(string(decodedResp))
	return nil
}

/*
// GetCmdGetContractHistory prints the code history for a given contract
func GetCmdGetContractHistory(cdc *codec.Codec) *cobra.Command {
	return &cobra.Command{
		Use:   "contract-history [bech32_address]",
		Short: "Prints out the code history for a contract given its address",
		Long:  "Prints out the code history for a contract given its address",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			cliCtx := context.NewCLIContext().WithCodec(cdc)

			addr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			route := fmt.Sprintf("custom/%s/%s/%s", types.QuerierRoute, keeper.QueryContractHistory, addr.String())
			res, _, err := cliCtx.Query(route)
			if err != nil {
				return err
			}
			fmt.Println(string(res))
			return nil
		},
	}
}
*/

type argumentDecoder struct {
	// dec is the default decoder
	dec                func(string) ([]byte, error)
	asciiF, hexF, b64F bool
}

func newArgDecoder(def func(string) ([]byte, error)) *argumentDecoder {
	return &argumentDecoder{dec: def}
}

func (a *argumentDecoder) RegisterFlags(f *flag.FlagSet, argName string) {
	f.BoolVar(&a.asciiF, "ascii", false, "ascii encoded "+argName)
	f.BoolVar(&a.hexF, "hex", false, "hex encoded  "+argName)
	f.BoolVar(&a.b64F, "b64", false, "base64 encoded "+argName)
}

func (a *argumentDecoder) DecodeString(s string) ([]byte, error) {
	found := -1
	for i, v := range []*bool{&a.asciiF, &a.hexF, &a.b64F} {
		if !*v {
			continue
		}
		if found != -1 {
			return nil, errors.New("multiple decoding flags used")
		}
		found = i
	}
	switch found {
	case 0:
		return asciiDecodeString(s)
	case 1:
		return hex.DecodeString(s)
	case 2:
		return base64.StdEncoding.DecodeString(s)
	default:
		return a.dec(s)
	}
}

func asciiDecodeString(s string) ([]byte, error) {
	return []byte(s), nil
}
