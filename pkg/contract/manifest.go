// Package contract defines the documents exchanged across usuyo's
// boundaries: the content package manifest (manifest.json), the adapter
// fetch output and the adapter declaration (adapter.yaml). See docs/SPEC.md
// §3 and §8.
//
// The JSON Schemas under schema/ are the normative form; the Go types are
// one binding and the round-trip tests keep both in step. Arrays in these
// documents are always present and may be empty.
//
// This directory is licensed under MIT (see LICENSE here) so adapters and
// SDKs outside the kernel can depend on it.
package contract

import "time"

// SchemaVersion is the current value of the manifest "schema" field.
const SchemaVersion = 1

// Kind classifies what a content package holds.
type Kind string

// Kind values (SPEC §3).
const (
	KindArticle Kind = "article"
	KindPost    Kind = "post"
	KindImage   Kind = "image"
	KindVideo   Kind = "video"
	KindFile    Kind = "file"
)

// Completeness records how much of the original an item has. It only rises.
type Completeness string

// Completeness values (SPEC §3).
const (
	CompletenessLink    Completeness = "link"
	CompletenessPartial Completeness = "partial"
	CompletenessFull    Completeness = "full"
)

// State is the existence state of an item (PRD §5 生命周期).
type State string

// State values (SPEC §3).
const (
	StateNormal  State = "normal"
	StateSlimmed State = "slimmed"
	StateTrashed State = "trashed"
)

// Entry names the entry point through which an item was captured.
type Entry string

// Entry values (SPEC §3).
const (
	EntryPaste     Entry = "paste"
	EntryExtension Entry = "extension"
	EntryMobile    Entry = "mobile"
	EntryImport    Entry = "import"
	EntrySync      Entry = "sync"
	EntryMCP       Entry = "mcp"
	EntryCLI       Entry = "cli"
	EntryAPI       Entry = "api"
)

// Action is what the user did with the content on its platform.
type Action string

// Action values (SPEC §3).
const (
	ActionFavorite Action = "favorite"
	ActionLike     Action = "like"
	ActionShare    Action = "share"
	ActionView     Action = "view"
	ActionUnknown  Action = "unknown"
)

// JobState summarizes one processing job on an item.
type JobState string

// JobState values as shown in SPEC §3.
const (
	JobPending JobState = "pending"
	JobDone    JobState = "done"
	JobFailed  JobState = "failed"
)

// Identity locates content on a platform (SPEC §7.1). Adapters return it from
// canonicalize; a manifest lists every identity the item is known by.
type Identity struct {
	Platform  string `json:"platform"`
	ContentID string `json:"contentId,omitempty"`
	URL       string `json:"url,omitempty"`
}

// Author is the content's author as the platform presents them.
type Author struct {
	Name       string `json:"name"`
	PlatformID string `json:"platformId,omitempty"`
	URL        string `json:"url,omitempty"`
}

// Origin records which entry point and platform action produced an item.
type Origin struct {
	Entry  Entry  `json:"entry"`
	Action Action `json:"action"`
	// ImportBatch is null outside batch imports.
	ImportBatch *string `json:"importBatch"`
}

// Snapshot is one fetch of the original, stored under RawDir.
type Snapshot struct {
	ID             string       `json:"id"`
	Adapter        string       `json:"adapter"`
	AdapterVersion string       `json:"adapterVersion"`
	At             time.Time    `json:"at"`
	Completeness   Completeness `json:"completeness"`
	RawDir         string       `json:"rawDir"`
}

// EngineTag is a tag suggested by a model or rule, shown to the user for
// confirmation only (SPEC §6).
type EngineTag struct {
	Tag        string    `json:"tag"`
	By         string    `json:"by"`
	At         time.Time `json:"at"`
	Confidence float64   `json:"confidence"`
}

// Tags keeps the three tag sources apart (SPEC §6).
type Tags struct {
	Platform []string    `json:"platform"`
	Engine   []EngineTag `json:"engine"`
	User     []string    `json:"user"`
}

// Fingerprints feed content-layer deduplication (SPEC §7.2).
type Fingerprints struct {
	Text   string   `json:"text,omitempty"`
	Images []string `json:"images"`
	Video  string   `json:"video,omitempty"`
}

// Manifest is manifest.json, the index card of a content package (SPEC §3).
type Manifest struct {
	Schema        int                 `json:"schema"`
	ID            string              `json:"id"`
	Identities    []Identity          `json:"identities"`
	Kind          Kind                `json:"kind"`
	Title         string              `json:"title"`
	Author        *Author             `json:"author,omitempty"`
	PublishedAt   *time.Time          `json:"publishedAt,omitempty"`
	CapturedAt    time.Time           `json:"capturedAt"`
	Origin        Origin              `json:"origin"`
	Completeness  Completeness        `json:"completeness"`
	Missing       []string            `json:"missing"`
	MissingReason string              `json:"missingReason,omitempty"`
	Snapshots     []Snapshot          `json:"snapshots"`
	Tags          Tags                `json:"tags"`
	Fingerprints  *Fingerprints       `json:"fingerprints,omitempty"`
	SameAs        []string            `json:"sameAs"`
	State         State               `json:"state"`
	Jobs          map[string]JobState `json:"jobs"`
}
