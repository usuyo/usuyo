import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ApiError, DaemonUnreachableError } from "../api/client";
import { ErrorLine } from "./ErrorLine";

afterEach(cleanup);

describe("ErrorLine", () => {
  it("shows what the desktop shell said, when it said something", () => {
    render(<ErrorLine error={new DaemonUnreachableError("x", "端口 47615 被别的程序占着，usuyo 的后台没能启动")} />);
    expect(screen.getByRole("status").textContent).toBe("端口 47615 被别的程序占着，usuyo 的后台没能启动");
  });

  it("shows the daemon's sentence for the user, not its English detail", () => {
    render(<ErrorLine error={new ApiError(400, "plugin: not an archive", "invalid-args", "这不像归档：里面没有 data/")} />);
    expect(screen.getByRole("status").textContent).toBe("这不像归档：里面没有 data/");
  });

  it("falls back to the calm default", () => {
    render(<ErrorLine error={new DaemonUnreachableError(new Error("x"))} />);
    expect(screen.getByRole("status").textContent).toBe("usuyo 服务未运行。重新打开 usuyo 后再试。");
  });
});
