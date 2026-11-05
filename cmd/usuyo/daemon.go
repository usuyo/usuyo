package main

import "github.com/spf13/cobra"

func newDaemonCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "daemon",
		Short: "常驻进程：任务队列、本地 API、文件监听",
		Args:  cobra.NoArgs,
		RunE:  notImplemented,
	}
}
