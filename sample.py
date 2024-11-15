import asyncio
import tlsn_langchain
import os
import json

from dotenv import load_dotenv
load_dotenv()

message ="""
        {
            \"role\": \"user\",
            \"content\": \"Hello, I am John, how are you doing?\"
        }
        """


async def main():
    result = await tlsn_langchain.exec("gpt-4o", os.getenv("REDPILL_API_KEY"), [message])
    print("Response: ", result[0])
    print("Proof: ", result[1])


# Run the async function
print("Running the async function")
asyncio.run(main())