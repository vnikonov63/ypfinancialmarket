[*] Add logging ability: for the server, for the client we can also do it but it has to be in a separate file, as we are using the terminal window for communication.
[ ] Add features, random stuff should be truly random, but if we do not want that it should use Hashing and the `SystemTime` thing 
[ ] Scan the whole repository for unwraps and other possible panics.
[ ] Add ability to provide a file for the client's tickers, that would be the default thing, and it would just allow you to write STREAM. The same logic should apply for the UDP clargument. If we write `STREAM 127.0.0.1:7879 AAPL,AMZN` use this. If we do the `STREAM` we should use the default clarguments.
[ ] Create a README
