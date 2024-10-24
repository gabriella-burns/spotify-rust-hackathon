# spotify-rust-hackathon

Today we’ll be building on an existing API framework to analyze Spotify listening data. 

There is an existing authorization flow in the auth.rs file. For the existing code to work, please log in to your Spotify account.

Your task today is to work in groups of 2 to modify the existing code so that it has the following end points:

[GET] /api/top-artists -> Displays top 50 artists
[GET] /api/top-artists/<genre> -> Displays top 50 artists in a genre
Extend this, so if genre is not real, returns sensible error message to page

Too easy for you? Try one of the following:
- Switch framework
- Try calling the Openai’s chatgpt api and asking it to roast or toast your music taste 

Documentation:
- Spotify Web API Documentation: https://developer.spotify.com/documentation/web-api
- OpenAI Documentation: https://platform.openai.com/docs/


