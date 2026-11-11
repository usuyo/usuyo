import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DaemonUnreachableError } from "../api/client";
import { ErrorLine } from "./ErrorLine";

afterEach(cleanup);

describe("ErrorLine", () => {
  it("shows what the desktop shell said, when it said something", () => {
    render(<ErrorLine error={new DaemonUnreachableError("x", "端口 47615 被别的程序占着，usuyo 的后台没能启动")} />);
    expect(screen.getByRole("status").textContent).toBe("端口 47615 被别的程序占着，usuyo 的后台没能启动");
  });

  it("falls back to the calm default", () => {
    render(<ErrorLine error={new DaemonUnreachableError(new Error("x"))} />);
    expect(screen.getByRole("status").textContent).toBe("内核没在运行");
  });
});
