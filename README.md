# Handoff

Handoff is based on the simple idea of "I have an artifact, and I need it deployed." A team will design one or more
standards based deployments: combining a metadata schema and deployer. CI systems will "handoff" any artifacts to the

It's meant to be used in tandem with a CI/CD system to handle the final five or ten percent of actual deployment.

## When might you need a system like this

If you have a large number of deployments, a tool like this will simplify your deployments by providing standards based
approaches.

If you exist in a regulated environment, standards based deployments are required anyway. You are likely reading this as
you are looking for a solution to complex compliance requirements like separations of duties.

## When you probably don't need a system like this

If you have five or less deployments, or you deploy once a quarter, this is a waste of your time. Maintaining this
system will cost you more than just taking screenshots and doing manual deployments.
