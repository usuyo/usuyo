package main

import "github.com/spf13/cobra"

func newAddCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "add <url|file|->",
		Short: "收下一条链接、文件或标准输入的文本；立即返回条目 id，后台获取",
		Args:  cobra.ExactArgs(1),
		RunE:  notImplemented,
	}
}
