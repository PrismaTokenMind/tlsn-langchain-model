import asyncio
import tlsn_langchain
import os

from dotenv import load_dotenv
# Load environment variables from .env file
load_dotenv()

message = {
       "role": "user",
       "content": "Hello, I am John, how are you doing?"
}


async def main():
    result = await tlsn_langchain.exec("gpt-4o", os.getenv("REDPILL_API_KEY"), list(message))
    print(result)


# Run the async function
print("Running the async function")
asyncio.run(main())