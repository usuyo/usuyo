package contract

import "time"

// Capability is an operation an adapter declares it supports.
type Capability string

// Capability values (SPEC §8).
const (
	CapabilityFetch Capability = "fetch"
	CapabilityList  Capability = "list"
	CapabilityWatch Capability = "watch"
)

// Method is how the kernel talks to an adapter.
type Method string

// Method values (SPEC §8).
const (
	MethodCLI          Method = "cli"
	MethodLocalService Method = "local-service"
	MethodMCP          Method = "mcp"
	MethodHTTPAPI      Method = "http-api"
	MethodBrowser      Method = "browser"
)

// Upstream names the project an adapter wraps and how to install it.
type Upstream struct {
	Project string `json:"project" yaml:"project"`
	License string `json:"license" yaml:"license"`
	// Install is free-form until SPEC §8 fixes its shape.
	Install map[string]any `json:"install,omitempty" yaml:"install,omitempty"`
}

// Requirements lists what an adapter needs from the user's environment.
type Requirements struct {
	Cookie string `json:"cookie" yaml:"cookie"`
	Proxy  string `json:"proxy" yaml:"proxy"`
}

// AdapterDecl is adapter.yaml, the declaration file of an adapter (SPEC §8).
type AdapterDecl struct {
	ID           string       `json:"id" yaml:"id"`
	Platforms    []string     `json:"platforms" yaml:"platforms"`
	Match        []string     `json:"match" yaml:"match"`
	Capabilities []Capability `json:"capabilities" yaml:"capabilities"`
	Method       Method       `json:"method" yaml:"method"`
	Upstream     *Upstream    `json:"upstream,omitempty" yaml:"upstream,omitempty"`
	Requires     Requirements `json:"requires" yaml:"requires"`
	Cost         string       `json:"cost" yaml:"cost"`
	Outputs      []string     `json:"outputs" yaml:"outputs"`
	Limits       string       `json:"limits,omitempty" yaml:"limits,omitempty"`
}

// Evidence records how a fetch was performed, for audit and retry.
type Evidence struct {
	FetchedAt time.Time `json:"fetchedAt"`
	// Status is the HTTP status code when the method has one.
	Status int `json:"status,omitempty"`
}

// FetchOutput is the JSON an adapter prints for `fetch <url> --out <dir>`
// (SPEC §8). HTML, Images and Media are paths relative to the --out directory.
type FetchOutput struct {
	Identity     Identity     `json:"identity"`
	Kind         Kind         `json:"kind"`
	Title        string       `json:"title"`
	Author       *Author      `json:"author,omitempty"`
	PublishedAt  *time.Time   `json:"publishedAt,omitempty"`
	Text         string       `json:"text"`
	HTML         string       `json:"html,omitempty"`
	Images       []string     `json:"images"`
	Media        []string     `json:"media"`
	PlatformTags []string     `json:"platformTags"`
	Completeness Completeness `json:"completeness"`
	Missing      []string     `json:"missing"`
	Evidence     Evidence     `json:"evidence"`
}
