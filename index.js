/*** NetworkWatch Z-Way HA module *******************************************

Version: 1.00
(c) Maroš Kollár, 2015-2017
-----------------------------------------------------------------------------
Author: Maroš Kollár <maros@k-1.com>
Description:
    Emit events based on network ARP packets

******************************************************************************/

function NetworkWatch (id, controller) {
    // Call superconstructor first (AutomationModule)
    NetworkWatch.super_.call(this, id, controller);
}

inherits(NetworkWatch, BaseModule);

_module = NetworkWatch;

// ----------------------------------------------------------------------------
// --- Module instance initialized
// ----------------------------------------------------------------------------

NetworkWatch.prototype.init = function (config) {
    NetworkWatch.super_.prototype.init.call(this, config);

    var command = '/opt/z-way-server/automation/' +
        self.moduleBasePath() +
        '/network_watch ' +
        config.interface +
        ' '+
        config.mac.join(' ');
    self.log('Startup "'+command+'"');
    system(command);

    // Setup global http
    NetworkWatchHandler = function(request) {
        self.log('Got watch event');
        console.log(request);
        var mac = request;
        self.controller.emit('networkwatch.arp',mac);
        return {
            status: 201,
            headers: {
                'Content-Type': 'text/plain',
            },
            body: '',
        };
    };

    ws.allowExternalAccess('NetworkWatchHandler',self.controller.auth.ROLE.LOCAL);
};

NetworkWatch.prototype.stop = function () {
    var self = this;
    NetworkWatch.super_.prototype.stop.call(this);
};

// ----------------------------------------------------------------------------
// --- Module methods
// ----------------------------------------------------------------------------