# Phantom

## Phantom proxy - A Rust MVP project

### Intro

The basic idea is to have some sort of "firewall" to handle security checks before the application layer.  
Basically, every request has to pass through an additional step before even reaching whatever app or framework is forwarded to.  
This approach allows us to perform security verifications and controls more efficiently, without affecting the next layer at all.

This MVP, in a very simple way, focuses on checking whether the current request path is found within a list of paths typically used by automatic scanning tools such as [Nikto](https://www.kali.org/tools/nikto/) or [Burpsuite](https://www.kali.org/tools/burpsuite/).  
When a request fails the check it is blocked and a 401 HTTP message is returned to the client.
