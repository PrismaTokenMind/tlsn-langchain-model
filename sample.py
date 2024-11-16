import asyncio
import tlsn_langchain
import os
import json

from dotenv import load_dotenv
load_dotenv()

messages =[
   """{
        \"role\": \"user\",
        \"content\": \"hi im bob! and i live in sf\"
    }""",
    """{
        \"role\": \"assistant\",
        \"content\": \"Hi Bob! It's great to meet you. How can I assist you today?\"
    }""",
    """{
        \"role\": \"user\",
        \"content\": \"whats the weather where I live?\"
    }"""
]

tools = [
    """
        {
            \"type\": \"function\",
            \"function\": {
                \"name\": \"tavily_search_results_json\",
                \"description\": \"A search engine optimized for comprehensive, accurate, and trusted results. Useful for when you need to answer questions about current events. Input should be a search query.\",
                \"parameters\": {
                    \"properties\": {
                        \"query\": {
                            \"description\": \"search query to look up\",
                            \"type\": \"string\"
                        }
                    },
                    \"required\": [\"query\"],
                    \"type\": \"object\"
                }
            }
        }
    """
]

top_p = 0.85
temperature = 0.3
stream = False



async def main():
    result = await tlsn_langchain.exec("gpt-4o", os.getenv("REDPILL_API_KEY"), messages, tools, top_p, temperature, stream)
    print("Response: ", result[0])
    print("Proof:", result[1].replace("\n", "").replace(" ", ""))

# Run the async function
print("Running the async function")
asyncio.run(main())