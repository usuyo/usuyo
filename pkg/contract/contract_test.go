package contract_test

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"github.com/goccy/go-yaml"
	"github.com/santhosh-tekuri/jsonschema/v6"

	"github.com/xiecun-app/xiecun/pkg/contract"
)

func TestManifestRoundTrip(t *testing.T) {
	for _, path := range samples(t, "manifest.*.json") {
		t.Run(filepath.Base(path), func(t *testing.T) {
			roundTripJSON[contract.Manifest](t, readFile(t, path), contract.ValidateManifest)
		})
	}
}

func TestFetchOutputRoundTrip(t *testing.T) {
	for _, path := range samples(t, "fetch-output.*.json") {
		t.Run(filepath.Base(path), func(t *testing.T) {
			roundTripJSON[contract.FetchOutput](t, readFile(t, path), contract.ValidateFetchOutput)
		})
	}
}

func TestAdapterDeclRoundTrip(t *testing.T) {
	for _, path := range samples(t, "adapter.*.yaml") {
		t.Run(filepath.Base(path), func(t *testing.T) {
			data := readFile(t, path)
			if err := contract.ValidateAdapterDecl(data); err != nil {
				t.Fatalf("sample rejected by schema: %v", err)
			}
			var decl contract.AdapterDecl
			if err := yaml.UnmarshalWithOptions(data, &decl, yaml.DisallowUnknownField()); err != nil {
				t.Fatalf("decode YAML into AdapterDecl: %v", err)
			}
			out, err := yaml.Marshal(decl)
			if err != nil {
				t.Fatalf("encode AdapterDecl as YAML: %v", err)
			}
			if err := contract.ValidateAdapterDecl(out); err != nil {
				t.Fatalf("re-encoded YAML rejected by schema: %v\n%s", err, out)
			}
			assertSameJSON(t, yamlToJSON(t, data), yamlToJSON(t, out))

			// The JSON binding of the same declaration must round-trip too.
			roundTripJSON[contract.AdapterDecl](t, yamlToJSON(t, data), contract.ValidateAdapterDecl)
		})
	}
}

func TestValidateRejectsInvalidDocuments(t *testing.T) {
	manifest := readFile(t, "testdata/manifest.full.json")
	fetch := readFile(t, "testdata/fetch-output.full.json")
	adapter := readFile(t, "testdata/adapter.full.yaml")

	cases := []struct {
		name     string
		validate func([]byte) error
		doc      []byte
	}{
		{"manifest: unknown kind", contract.ValidateManifest, withField(t, manifest, "kind", "podcast")},
		{"manifest: unknown field", contract.ValidateManifest, withField(t, manifest, "unread", 3)},
		{"manifest: bad timestamp", contract.ValidateManifest, withField(t, manifest, "capturedAt", "2026-10-20")},
		{"manifest: unknown job state", contract.ValidateManifest, withField(t, manifest, "jobs", map[string]any{"ocr": "running"})},
		{"manifest: missing required", contract.ValidateManifest, withoutField(t, manifest, "snapshots")},
		{"fetch: unknown completeness", contract.ValidateFetchOutput, withField(t, fetch, "completeness", "most")},
		{"fetch: missing evidence", contract.ValidateFetchOutput, withoutField(t, fetch, "evidence")},
		{"fetch: not an array", contract.ValidateFetchOutput, withField(t, fetch, "images", nil)},
		{"adapter: unknown method", contract.ValidateAdapterDecl, bytes.ReplaceAll(adapter, []byte("method: cli"), []byte("method: telepathy"))},
		{"adapter: unknown field", contract.ValidateAdapterDecl, append(adapter, []byte("\nbin: ./fetch\n")...)},
		{"adapter: malformed id", contract.ValidateAdapterDecl, bytes.ReplaceAll(adapter, []byte("community/wechat-mp-basic"), []byte("WeChat"))},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			if err := tc.validate(tc.doc); err == nil {
				t.Fatalf("expected a validation error\n%s", tc.doc)
			}
		})
	}
}

// Identity, author, kind and completeness are duplicated between the manifest
// and fetch-output schemas so each file validates on its own; they must not
// drift apart.
func TestSharedDefsIdentical(t *testing.T) {
	manifestDefs := schemaDefs(t, contract.ManifestSchema)
	fetchDefs := schemaDefs(t, contract.FetchOutputSchema)
	for _, def := range []string{"identity", "author", "kind", "completeness"} {
		if !reflect.DeepEqual(manifestDefs[def], fetchDefs[def]) {
			t.Errorf("$defs/%s differs between manifest and fetch-output schemas", def)
		}
	}
}

// roundTripJSON checks that data passes validate, decodes strictly into T,
// re-encodes to a document that still validates and equals the input.
func roundTripJSON[T any](t *testing.T, data []byte, validate func([]byte) error) {
	t.Helper()
	if err := validate(data); err != nil {
		t.Fatalf("sample rejected by schema: %v", err)
	}
	var v T
	dec := json.NewDecoder(bytes.NewReader(data))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&v); err != nil {
		t.Fatalf("decode into %T: %v", v, err)
	}
	out, err := json.Marshal(v)
	if err != nil {
		t.Fatalf("encode %T: %v", v, err)
	}
	if err := validate(out); err != nil {
		t.Fatalf("re-encoded %T rejected by schema: %v\n%s", v, err, out)
	}
	assertSameJSON(t, data, out)
}

func assertSameJSON(t *testing.T, want, got []byte) {
	t.Helper()
	if w, g := decodeAny(t, want), decodeAny(t, got); !reflect.DeepEqual(w, g) {
		t.Errorf("round trip changed the document\nwant: %s\ngot:  %s", want, got)
	}
}

// decodeAny keeps numbers as json.Number so 0.82 and 200 compare exactly.
func decodeAny(t *testing.T, data []byte) any {
	t.Helper()
	v, err := jsonschema.UnmarshalJSON(bytes.NewReader(data))
	if err != nil {
		t.Fatalf("decode JSON: %v\n%s", err, data)
	}
	return v
}

func yamlToJSON(t *testing.T, data []byte) []byte {
	t.Helper()
	out, err := yaml.YAMLToJSON(data)
	if err != nil {
		t.Fatalf("convert YAML to JSON: %v", err)
	}
	return out
}

func withField(t *testing.T, data []byte, key string, value any) []byte {
	t.Helper()
	doc := decodeObject(t, data)
	doc[key] = value
	return encode(t, doc)
}

func withoutField(t *testing.T, data []byte, key string) []byte {
	t.Helper()
	doc := decodeObject(t, data)
	delete(doc, key)
	return encode(t, doc)
}

func decodeObject(t *testing.T, data []byte) map[string]any {
	t.Helper()
	var doc map[string]any
	if err := json.Unmarshal(data, &doc); err != nil {
		t.Fatalf("decode JSON object: %v", err)
	}
	return doc
}

func encode(t *testing.T, v any) []byte {
	t.Helper()
	out, err := json.Marshal(v)
	if err != nil {
		t.Fatalf("encode JSON: %v", err)
	}
	return out
}

func schemaDefs(t *testing.T, name string) map[string]any {
	t.Helper()
	raw, err := contract.Schemas.ReadFile(name)
	if err != nil {
		t.Fatalf("read %s: %v", name, err)
	}
	defs, ok := decodeObject(t, raw)["$defs"].(map[string]any)
	if !ok {
		t.Fatalf("%s has no $defs object", name)
	}
	return defs
}

func samples(t *testing.T, pattern string) []string {
	t.Helper()
	paths, err := filepath.Glob(filepath.Join("testdata", pattern))
	if err != nil {
		t.Fatalf("glob %s: %v", pattern, err)
	}
	if len(paths) == 0 {
		t.Fatalf("no samples match testdata/%s", pattern)
	}
	return paths
}

func readFile(t *testing.T, path string) []byte {
	t.Helper()
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read %s: %v", path, err)
	}
	if !strings.HasSuffix(path, ".yaml") && !json.Valid(data) {
		t.Fatalf("%s is not valid JSON", path)
	}
	return data
}
