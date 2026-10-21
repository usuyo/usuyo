package main

import (
	"io"
	"slices"
	"strings"
	"testing"
)

// specCommands is the subcommand list of SPEC §9.
var specCommands = []string{"add", "daemon", "get", "init", "jobs", "mcp", "reindex", "search", "tag"}

func TestSubcommandsMatchSpec(t *testing.T) {
	var got []string
	for _, c := range newRootCmd().Commands() {
		got = append(got, c.Name())
	}
	slices.Sort(got)
	if !slices.Equal(got, specCommands) {
		t.Fatalf("subcommands = %v, want %v", got, specCommands)
	}
}

func TestSubcommandsReportNotImplemented(t *testing.T) {
	cases := [][]string{
		{"init"},
		{"add", "https://example.com/a"},
		{"get", "web-0123abcd-k9x2", "--layer", "original"},
		{"search", "咖啡", "--src", "douyin", "--in", "title", "--kind", "video", "--json"},
		{"tag", "web-0123abcd-k9x2", "--add", "a,b", "--rm", "c"},
		{"jobs", "--retry", "42"},
		{"reindex"},
		{"daemon"},
		{"mcp"},
	}
	for _, args := range cases {
		t.Run(args[0], func(t *testing.T) {
			root := newRootCmd()
			root.SetOut(io.Discard)
			root.SetErr(io.Discard)
			root.SetArgs(args)
			err := root.Execute()
			if err == nil || !strings.Contains(err.Error(), "not implemented") {
				t.Fatalf("xiecun %s: err = %v, want not implemented", strings.Join(args, " "), err)
			}
		})
	}
}
