#include "server.h"

class Follwer: public Server {
public: 
    // send response to leader
    response_append_entries (bool success) {
        response_append_entries_RPC msg = {
            followerId: serverId,
            term: currentTerm,
            nextIndex: commitIndex + 1,
            matchIndex: log.back().index - 1,
            success: success
        };
        send(msg, leaderId);
    }
    
    // whether follower's log is more up_to_date than leader's
    bool more_uptodate(int lastLogTerm, int lastLogIndex) {
        if(lastLogTerm < currentTerm) return true;
        else if(lastLogTerm > currentTerm) return false;
        else {
            if(lastLogIndex <= log.back().index - 1) return true; // when "==" , follower will reject voting
            else return false;
        }
    }

private:
    int votedFor = 0;	// candidateId that received vote in current term, initialized to 0
    int leaderId;

    // vote_for should be a Singleton for each term
    void vote_for(int CandidateId, bool granted) {
        response_vote_RPC msg  = {
            term:           currentTerm,
            voteGranted:    granted
        }
        send(msg, CandidateId);
    }
};


//thread1:
on_timeout(election) {
    switch_to(CANDIDATE);
}

//thread2:
listen_on(request_from_client msg) {
    if(msg.write)
        /* redirect to leader */;
    else if (msg.read)
        /* response to client */; 
}

//thread3: 
listen_on(request_vote_RPC msg) {
    refresh_timeout(election);
    if (msg.term >= currentTerm 
    && (votedFor == msg.candidateId || votedFor == 0) 
    && !more_uptodate(msg.lastLogTerm, msg.lastLogIndex)) {
        currentTerm = msg.term;
        vote_for(msg.candidateId, true);
    }
    else {
        vote_for(msg.candidateId, false);
    }
}

//thread4:
listen_on(request_append_entries_RPC lmsg) {
    refresh_timeout(election);

    if(lmsg.entries != NULL) { 
        bool success;
        if (lmsg.term < currentTerm 
        || lmsg.prevLogIndex >= log.back().index 
        || lmsg.entries[prevLogIndex].term != log[prevLogIndex].term) {
            success = false;
        }
        else {
            log.push_back(lmsg.entries);
            if (lmsg.leaderCommit > commitIndex) 
                commitIndex = min(lmsg.leaderCommit, log.back().index - 1);
            success = true;
        }
        response_append_entries(success);
    }
}