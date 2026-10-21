package contract

import (
	"bytes"
	"embed"
	"fmt"
	"sync"

	"github.com/goccy/go-yaml"
	"github.com/santhosh-tekuri/jsonschema/v6"
)

// Schemas holds the JSON Schema (draft 2020-12) of each contract document.
//
//go:embed schema/*.schema.json
var Schemas embed.FS

// Paths of the schemas inside Schemas.
const (
	ManifestSchema    = "schema/manifest.schema.json"
	FetchOutputSchema = "schema/fetch-output.schema.json"
	AdapterDeclSchema = "schema/adapter.schema.json"
)

// ValidateManifest checks a manifest.json document against ManifestSchema.
func ValidateManifest(data []byte) error {
	return validateJSON(ManifestSchema, data)
}

// ValidateFetchOutput checks an adapter fetch output against FetchOutputSchema.
func ValidateFetchOutput(data []byte) error {
	return validateJSON(FetchOutputSchema, data)
}

// ValidateAdapterDecl checks an adapter.yaml document (YAML or JSON) against
// AdapterDeclSchema.
func ValidateAdapterDecl(data []byte) error {
	asJSON, err := yaml.YAMLToJSON(data)
	if err != nil {
		return fmt.Errorf("parse adapter.yaml: %w", err)
	}
	return validateJSON(AdapterDeclSchema, asJSON)
}

func validateJSON(name string, data []byte) error {
	doc, err := jsonschema.UnmarshalJSON(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("parse document for %s: %w", name, err)
	}
	schemas, err := compiledSchemas()
	if err != nil {
		return err
	}
	if err := schemas[name].Validate(doc); err != nil {
		return fmt.Errorf("validate against %s: %w", name, err)
	}
	return nil
}

var (
	compileOnce sync.Once
	compiled    map[string]*jsonschema.Schema
	errCompile  error
)

// compiledSchemas compiles the embedded schemas once; formats such as
// date-time and uri are asserted, not just annotated.
func compiledSchemas() (map[string]*jsonschema.Schema, error) {
	compileOnce.Do(func() {
		compiled, errCompile = compileAll()
	})
	return compiled, errCompile
}

func compileAll() (map[string]*jsonschema.Schema, error) {
	c := jsonschema.NewCompiler()
	c.AssertFormat()
	out := make(map[string]*jsonschema.Schema)
	for _, name := range []string{ManifestSchema, FetchOutputSchema, AdapterDeclSchema} {
		raw, err := Schemas.ReadFile(name)
		if err != nil {
			return nil, fmt.Errorf("read embedded %s: %w", name, err)
		}
		doc, err := jsonschema.UnmarshalJSON(bytes.NewReader(raw))
		if err != nil {
			return nil, fmt.Errorf("parse %s: %w", name, err)
		}
		if err := c.AddResource(name, doc); err != nil {
			return nil, fmt.Errorf("register %s: %w", name, err)
		}
		sch, err := c.Compile(name)
		if err != nil {
			return nil, fmt.Errorf("compile %s: %w", name, err)
		}
		out[name] = sch
	}
	return out, nil
}
