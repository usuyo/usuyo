// Command usuyo is the CLI entry point. It also starts the daemon and the
// stdio MCP server as subcommands (SPEC §9).
package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

// version is overridden at build time via -ldflags "-X main.version=...".
var version = "dev"

func main() {
	if err := newRootCmd().Execute(); err != nil {
		fmt.Fprintln(os.Stderr, "usuyo:", err)
		os.Exit(1)
	}
}

func newRootCmd() *cobra.Command {
	root := &cobra.Command{
		Use:           "usuyo",
		Short:         "usuyo：在推荐流外，维护一套你自己的思路",
		Version:       version,
		SilenceUsage:  true,
		SilenceErrors: true,
	}
	root.AddCommand(
		newInitCmd(),
		newAddCmd(),
		newGetCmd(),
		newSearchCmd(),
		newTagCmd(),
		newJobsCmd(),
		newReindexCmd(),
		newDaemonCmd(),
		newMCPCmd(),
	)
	return root
}

// notImplemented is the RunE of every subcommand until its feature lands.
func notImplemented(cmd *cobra.Command, _ []string) error {
	return fmt.Errorf("%s: not implemented", cmd.Name())
}
