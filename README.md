# Fourier Fit
## About

## Development Notes
Developing for this repository is recommended to be done in a devcontainer. To so do, ensure that `docker` is installed, and an ssh agent is running. For more information, visit the [credential sharing page](https://code.visualstudio.com/remote/advancedcontainers/sharing-git-credentials). However, this can be broken into specific steps that are highly important to ensure seamless development.

### Authentication and Signing Keys
First, create an `ed25519` key by running `ssh-keygen -t ed25519`. This should be done twice--once to generate an authetication key and once to generate a signing key (you must specify a different file name for each). Then, add the keys to GitHub, specifying the authentication and signing keys.

### Configuring git
`git` must be installed and should be configured globally to ensure commits can be created before pushing to the remote repo. Specifically, specify the user's name and email by running the following commands and tweaking appropriately.
```
git config --global user.name "Your Name"
git config --global user.email "Your Email"
```
Then, configure signing with the ssh signing key created in the prior step with the following commands.
```
git config --global gpg.format ssh
git config --global user.signingkey <path-to-public-signing-key>
git config --global commit.gpgsign true
```
Then, in `~/.bashrc`, append the following:
```
if [ -z "$SSH_AUTH_SOCK" ]; then
   # Check for a currently running instance of the agent
   RUNNING_AGENT="`ps -ax | grep 'ssh-agent -s' | grep -v grep | wc -l | tr -d '[:space:]'`"
   if [ "$RUNNING_AGENT" = "0" ]; then
        # Launch a new instance of the agent
        ssh-agent -s &> $HOME/.ssh/ssh-agent
   fi
   eval `cat $HOME/.ssh/ssh-agent` > /dev/null
   ssh-add <path-to-private-authentication-key> <path-to-private-signing-key> 2> /dev/null
fi
```
However, since there are authentication and signing keys now, configure git to use the authentication key for authentication by opening `~/.ssh/config` in a text editor and adding:
```
Host github.com
    User git
    IdentityFile <path-to-private-authentication-key>
    IdentitiesOnly yes
```
### Configuring the ssh agent
An ssh agent must be running and forwarded into the devcontainer. Check if it is running with `eval "$(ssh-agent -s)"` and check the existence of a socket with `echo $SSH_AUTH_SOCK`. If both return a good status, open `/etc/ssh/sshd_config` in text editor and add/uncomment the line `AllowAgentForwarding yes`. After following these steps, devcontainers should be able to allow commits and pushes to the remote from within the container itself.
