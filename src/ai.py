from langchain_google_genai import ChatGoogleGenerativeAI
from langchain_core.prompts import PromptTemplate, MessagesPlaceholder
from langchain.memory import ConversationBufferMemory
from langchain.memory import ConversationBufferWindowMemory
from langchain_core.messages import HumanMessage, AIMessage, SystemMessage
from langchain_core.messages import trim_messages
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import START, MessagesState, StateGraph
from dotenv import load_dotenv, find_dotenv
import os 
import getpass

load_dotenv(find_dotenv(), override = True)
if 'GOOGLE_API_KEY' not in os.environ:
   os.environ['GOOGLE_API_KEY'] = getpass.getpass('Provide your Google API Key: ')
# GEMINI_API_KEY = os.environ.get("GOOGLE_API_KEY")
GEMINI_API_KEY = os.getenv("GOOGLE_API_KEY")

llm = ChatGoogleGenerativeAI(model = "gemini-1.5-flash-002", google_api_key = GEMINI_API_KEY)

# PROMPT 1: determines if an action specified by the user is similar to any in a set list of actions
verbPrompt = PromptTemplate(
   input_variables = ["actions", "question"],
   template = "Is the prompt from {question} similar to any of the actions specified in the {actions} list? If yes, which one (respond without quotation marks)? If none are similar, respond with 'None'."
)

# PROMPT 2: embellishes a short scene description
descriptionPrompt = PromptTemplate(
   input_variables=["description"],
   template="You are a GM for a cozy, but secretly dangerous, high fantasy setting. Embellish the scene described in {description}, but please try and keep the important details the same. Keep your response to one paragraph."
)

# PROMPT 3: provides a description of a room the player is in based on how their action changes the environment, or a quirky response
newRoomPrompt = PromptTemplate(
   input_variables = ["items", "location", "description", "question"],
   template =  "The player is located in the {location}. The {location} is described as {description}. The {location} contains {items}. Given this context, does the player's action {question} make sense? If so, describe the resulting scene after the action is completed with a short, whimsical sentence. Otherwise, respond with a short passive aggressive statement explaining why the player cannot do {question} based on the provided context."
)

# ------------------------------

# for the memory-associated details of Prompt 3:
# Define trimmer
# count each message as 1 "token" (token_counter=len) and keep only the last two messages
trimmer = trim_messages(strategy="last", max_tokens=3, token_counter=len)

# define a new workflow
workflow = StateGraph(state_schema=MessagesState)

# Define the function that calls the model
def call_model(state: MessagesState):
    trimmed_messages = trimmer.invoke(state["messages"])
    system_prompt = (
        "You are a GM for a fantasy adventure."
        "Try not to be too meta and try not to break the 4th wall in your messages."
        "Your general tone should be somewhat lighthearted."
        "When using previous message history as an input, most recent human message should have the greatest impact on your description except if you previously denied the instruction"
        "Generally, you will be providing descriptions of rooms after the user does specified actions"
        "If the history provided in {trimmed_messages} shows the action the user wants to do has already been done previously, include this in your response."
    )
    messages = [SystemMessage(content=system_prompt)] + trimmed_messages
    response = llm.invoke(messages)
    return {"messages": response}


# Define the (single) node in the graph
workflow.add_edge(START, "model")
workflow.add_node("model", call_model)

# Add memory
memory = MemorySaver()
app = workflow.compile(checkpointer=memory)

# ------------------------------

# functions for calling each different prompt

def generateAction(actions, question):
    prompt = verbPrompt.format(actions=actions, question=question)
    response = llm.invoke(prompt)
    return response.content


def generateDescription(_description):
    prompt = descriptionPrompt.format(description = _description)
    response = llm.invoke(prompt)
    return response.content

def generateScene(_items, _location, _description, _question, _history):
    prompt = newRoomPrompt.format(items=_items, location=_location, description=_description, question=_question)

    response = app.invoke(
        {
            "messages": _history
            + [HumanMessage(content=prompt)]
        },
        config={"configurable": {"thread_id": "2"}},
    )

    return response["messages"][-1].content