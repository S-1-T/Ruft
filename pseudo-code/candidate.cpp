#include "server.h"

class Candidate: public Server {
// exit when:
// 1.canditate win
// 2.another server becomes leader and send heartbeat
// 3.a peroid of time goes by with no winner
public:
    Candidate() {
        refresh_timeout(election);
        currentTerm ++; 
        votes = 1;        // vote for self
        request_vote();
    }

    //in parallel , periodically
    request_vote() {
        request_vote_RPC msg = {
            term: currentTerm,
            candidateId: serverId,
            lastLogIndex: log.back().index,
            lastLogTerm:  log.back().term
        }; 
        send(msg, others) 
    }

private:
    int votes;         //the number of votes that candidate gets
};


//thread1:
listen_on(response_vote_RPC msg) {
    if(msg.term > currentTerm) {
        currentTerm = msg.term;
        switch_to(FOLLOWER);
    }
    else if(msg.voteGranted) {
        votes++;
        if(votes > MAJORITY) switch_to(LEADER);
    }   
}

//thread2:
listen_on(request_append_entries_RPC msg) {
    if(msg.term >= currentTerm) {
        currentTerm = msg.term;
        switch_to(FOLLOWER);
    }
}

//thread3:
on_timeout(election) {
    currentTerm++;
    request_vote();
}