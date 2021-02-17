# Toriage
Toriage is a dashboard prototype for triage, and inspired by [homu](https://github.com/rust-lang/homu). This is under development.

## Usage
`cargo run` and access to `http://127.0.0.1:8080`.

PRs are arranged in order of the latest update date. If the PR has the `S-waiting-on-author` label, `Author` column is colored. If the PR has the `S-waiting-on-review` label, `Assignee` column is colored. After 1 week it gets yellow, after 2 weeks it gets red. If it's green, that's fine.
