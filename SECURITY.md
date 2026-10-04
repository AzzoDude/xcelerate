# Security Policy

Xcelerate drives real browsers, executes plugin code, and ships native code into
many language ecosystems, so security is treated as a first-class concern. This
policy explains which versions are supported and how to report a vulnerability.

## Supported Versions

Security fixes are provided for the latest release line only. Please upgrade to
the newest patch release before reporting.

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

Bindings published from the same workspace version (Rust, Python, JavaScript,
.NET, Kotlin, Java, Swift, Ruby, Dart, Go) share one release line; a fix is
shipped to all targets at once.

## Reporting a Vulnerability

**Please do not open a public issue for security problems.**

Use GitHub's private vulnerability reporting:

1. Go to the repository's **Security** tab.
2. Click **Report a vulnerability**.
3. Include the details below.

If you cannot use GitHub, email the maintainer at
`security@chaoswarehq.com` and we will open a private advisory on your behalf.

### What to include

- Affected version(s) and binding/language.
- A description of the issue and its impact.
- Minimal reproduction steps or a proof of concept.
- Whether you believe the issue is already public.
- Your name/handle for credit (optional).

### What to expect

| Stage | Target |
| --- | --- |
| Acknowledgement of your report | within 3 business days |
| Initial assessment (severity, scope) | within 7 business days |
| Fix / mitigation for accepted issues | as soon as practical, coordinated with you |

We will keep you informed of progress and credit you in the advisory unless you
ask us not to. Please give us a reasonable window to release a fix before any
public disclosure.

## Scope

In scope:

- **The Rust core and facade** (`xcelerate-core`, `xcelerate`,
  `xcelerate-plugin-api`, `xcelerate-plugins`).
- **The plugin trust model** - anything that lets a plugin exceed its granted
  capabilities, bypass manifest validation, escape the default-deny rules, or
  tamper with the append-only audit log.
- **The sandbox / capability proxy** once implemented - escapes or confused-deputy
  issues in the out-of-process runner.
- **The language bindings** (UniFFI-generated) and their packaging, where the
  vulnerability is in the binding/host, not the target language.
- **Supply chain** - compromised release artifacts, build scripts, or the
  generator pipeline.

Out of scope:

- **The `stealth` plugin's intended behaviour.** Reducing common automation
  fingerprints is a documented feature, not a vulnerability.
- **Third-party plugin code.** How a third-party plugin behaves is the author's
  responsibility and is used at your own risk. Report flaws in a *specific
  plugin* to that plugin's author. A flaw in the **host** that grants a plugin
  more than it asked for is in scope.
- Vulnerabilities in Chrome, Edge, or the operating system - report those
  upstream.
- Social engineering, physical access, or issues requiring an already-compromised
  machine.

## Safe Harbor

We will not pursue or support legal action against researchers who:

- make a good-faith effort to follow this policy;
- avoid privacy violations, data destruction, and service disruption;
- only interact with systems they own or are authorized to test; and
- give us a reasonable opportunity to fix the issue before disclosure.

Thank you for helping keep Xcelerate and its users safe.
