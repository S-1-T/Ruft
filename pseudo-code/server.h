#include "RPC.h"
#include <vector>

#define TOTAL 5        // there are 5 servers
#define MAJORITY 3     // 5 / 2 + 1

//mode of server
typedef enum {
    FOLLOWER,
    CANDIDATE,
    LEADER
}STATE;

// class Server provides the basic function and data structure of a server
class Server {
public:
    Server() {
        /* 
        initial_ID();    
        initial_log(); 
        */
    }

    virtual listen_on(RPC) {
        /* if receive RPC request or RPC response, do something here */
    }

    virtual send(RPC, ID) {
        /* send message to server which serverId == ID */
    }

    virtual on_timeout(clock_name) {
        /* if clock time out, do something here */
    }

    virtual refresh_timeout(clock_name) {
        /* reset clock */
    }

    switch_to(STATE state) {
        /* switch to a new state */
    }

protected:
    int serverId;
    int currentTerm;  	//latest term server has seen
    int commitIndex;    //index of highest log entry known to be committed
    int lastApplied;    //index of highest log entry applied to state machine
    vector<LOG> log;    //log entries
};


//thread0: triggered by increasement of commitIndex
append_entries() {
    while (lastApplied < commitIndex) {
        /* append log[lastApplied] to state machine */
        lastApplied ++;
    }
}

