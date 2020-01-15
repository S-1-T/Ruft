#include <string.h>


// the structure of log entry
typedef struct {
    // bool committed; //true if the entry is committed
    // int appended;   //the number of servers which has appended this entry
    int term;       // term number
    int index;      // index in log
	string command;	// operation that apply on state machine
}LOG;

//the structure of request_from_client, from client to server
typedef struct {
    bool write;
    bool read;
    string command;
}request_from_client;

//the structure of request_vote_RPC, from candidate to follower
typedef struct {
    int term;
    int candidateId;
    int lastLogIndex;
    int lastLogTerm;
}request_vote_RPC;

//the structure of response_vote_RPC, from follower to candidate
typedef struct {
    int term;           // currentTerm, for candidate to update itself
    bool voteGranted;   // true means candidate received vote
}response_vote_RPC;

//the structure of request_append_entries_RPC, from leader to followers
typedef struct {
   	int term;
   	int leaderId;
    int prevLogIndex;   //leader's log.back().index
    int prevLogTerm;    //leader's log.back().term
    int leaderCommit;   //leader's index of highest log entry known to be committed
    LOG entries[];      // entries need to be replicated
}request_append_entries_RPC;

//the structure of response_append_entries_RPC, from follower to leader
typedef struct {
    int followerId;
    int term;
    int nextIndex;  
    int matchIndex; 
    bool success;
}response_append_entries_RPC;