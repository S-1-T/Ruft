#include "server.h"

class Leader: public Server {
public:
    Leader() {
        request_append_entries_RPC msg = {
            term:           currentTerm,
            leaderId:       serverId,
            prevLogIndex:   NULL,
            prevLogTerm:    NULL,
            leaderCommit:   NULL,
            entries:        NULL       //carry no log entries
        };
        send(msg, others); 
        /* initial nextIndex[] to leader's last log index + 1,
           initial matchIndex[] to all 0s */
    }

private:
    int nextIndex[TOTAL];   //for each server, index of the next log entry to send 
    int matchIndex[TOTAL];  //for each server, index of highest log entry known to be replicated on server
};


//thread1:
on_timeout(heartbeat)
    request_append_entries_RPC msg = {
        term:           currentTerm,
        leaderId:       serverId,
        prevLogIndex:   NULL,
        prevLogTerm:    NULL,
        leaderCommit:   NULL,
        entries:        NULL
    };
    send(msg, others); //carry no log entries
}

//thread2:
listen_on(request_from_client msg) {
    log.push_back({currentTerm, msg.command});
    /* wait until new entry apply to state machine */
    /* response_to_client() */
}

//thread3:
check_append() {
    if(log.back().index >= nextIndex[followerId] && followerId != serverId) // assume followerId: 1..5
    request_append_entries_RPC msg = {
        term:           currentTerm,
        leaderId:       serverId,
        prevLogIndex:   log.back().index,
        prevLogTerm:    log.back().term,
        leaderCommit:   commitIndex,
        entries:        log[nextIndex[followerId]..back]
    };
    send(msg, followerId);
}

//thread4:
listen_on(response_append_entries_RPC msg) {
    if(msg.success) {
        nextIndex[msg.followerId] = msg.nextIndex;
        matchIndex[msg.followerId] = msg.matchIndex;
    }
    else {
        if(currentTerm < msg.term) switch_to(FOLLOWER);
        else {
            nextIndex[msg.followerId] -- ;
        }
    }
}

//thread5:
update_commitIndex() {
    // If there exists an N such that N > commitIndex, a majority
    // of matchIndex[i] ≥ N, and log[N].term == currentTerm:
    // set commitIndex = N 
    int N = commitIndex;
    while(true) {
        int match = 0;
        for (int i = 0; i < TOTAL; i++) {
            if (matchIndex[i] >= N && log[N].term == currentTerm){
                match ++;
            }
        }
        if (match >= MAJORITY) N++;
        else break;
    }
    commitIndex = N;
}
