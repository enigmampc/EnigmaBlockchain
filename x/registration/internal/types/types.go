package types

import (
	"encoding/base64"
	"encoding/hex"
	ra "github.com/enigmampc/EnigmaBlockchain/x/registration/remote_attestation"
)

const PublicKeyLength = 128   // encoded length
const EncryptedKeyLength = 96 // encoded length
const MasterPublicKeyId = "MasterKey"
const SecretNodeSeedConfig = "seed.json"
const SecretNodeCfgFolder = ".node"

const AttestationCertPath = "attestation_cert"

type NodeID []byte

// User struct which contains a name
// a type and a list of social links
type SeedConfig struct {
	MasterCert   string `json:"pk"`
	EncryptedKey string `json:"encKey"`
}

func (c SeedConfig) Decode() ([]byte, []byte, error) {
	enc, err := hex.DecodeString(c.EncryptedKey)
	if err != nil {
		return nil, nil, err
	}
	pk, err := base64.StdEncoding.DecodeString(c.MasterCert)
	if err != nil {
		return nil, nil, err
	}

	return pk, enc, nil
}

type RegistrationNodeInfo struct {
	Certificate   ra.Certificate
	EncryptedSeed []byte
}
