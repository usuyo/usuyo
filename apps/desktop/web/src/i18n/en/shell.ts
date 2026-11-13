import type zh from "../zh-Hans/shell";
import { mirror } from "../messages";

export default mirror<typeof zh>()({
  title: "Sign In to usuyo",
  visitor: "No sign-in needed here. The page creates a visitor identity when it opens, and the window closes once it's received. A visitor identity isn't your {site} account.",
  manual: "Sign in to {site} below. This window is used only for {site}, separate from other sites and the browser you usually use. When you're signed in, click Done Signing In.",
  auto: "Sign in to {site} below. This window is used only for {site}, separate from other sites and the browser you usually use. The window closes when you're signed in.",
  done: "Done Signing In",
});
