# fli
cli tool to list directory content

<img width="160" height="160" alt="mascot" src="https://github.com/user-attachments/assets/a0742301-5891-49a3-a746-29fa4b87eabc" />

# Why
I built it to use on my raspberry pi zero which I access through ssh. 
The purpose of this project is to build file-listing cli tool with a good readability, tiny binary size and maximum execution speed. 
Speed is priority: by default directory entries are streamed directly from `readdir()` to stdout without heap allocation.
Since rust `std` contributes heavily to binary size, this project is `no_std` + `libc` (**it contains unsafe code blocks**).

**Current binary size:**
- M series mac: ca 54kb,
- rpi zero w : ca 20kb.

Current scope of features is intentionally limited to simple retrieval of directory content without sorting and with natural sorting by name with -s flag. New features such as displaying metadata and sorting by size will be added soon.   



<img width="495" height="222" alt="macos" src="https://github.com/user-attachments/assets/7296dff8-2715-4cf2-aa62-05d5936dc59c" />

<img width="197" height="416" alt="rpi zero w" src="https://github.com/user-attachments/assets/d6979f22-55ec-4491-accd-08bc0b587b50" />
