package main

import "github.com/spf13/cobra"

func newMCPCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "mcp",
		Short: "以 stdio 提供 MCP 服务（转发到 daemon，daemon 不在则内嵌）",
		Args:  cobra.NoArgs,
		RunE:  notImplemented,
	}
}
